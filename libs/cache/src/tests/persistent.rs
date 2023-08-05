use std::{
    any::Any,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{de::DeserializeOwned, Serialize};
use test_log::test;
use tokio::runtime::Handle;

use crate::{
    error::{Error, Result},
    persistent::client::{
        create_runtime, create_server_and_clients, setup_test, ServerKind,
        TEST_SERVER_HEARTBEAT_TIMEOUT,
    },
    tests::Key,
    CacheHandle,
};

use crate::persistent::client::{Client, ClientKind};

pub(crate) const BASIC_TEST_NAMESPACE: &str = "test";
pub(crate) const BASIC_TEST_PARAM: (u64, u64) = (3, 5);
pub(crate) const BASIC_TEST_GENERATE_FN: fn(&(u64, u64)) -> u64 = tuple_sum;
pub(crate) const BASIC_TEST_ALT_NAMESPACE: &str = "test_alt";
pub(crate) const BASIC_TEST_ALT_GENERATE_FN: fn(&(u64, u64)) -> u64 = tuple_multiply;

pub(crate) fn cached_generate<
    K: Serialize + Send + Sync + Any,
    V: Serialize + DeserializeOwned + Send + Sync + Any,
>(
    client: &Client,
    duration: Option<Duration>,
    count: Option<Arc<Mutex<u64>>>,
    namespace: impl Into<String>,
    key: K,
    generate_fn_inner: impl FnOnce(&K) -> V + Send + Any,
) -> CacheHandle<V> {
    client.generate(namespace, key, move |k| {
        if let Some(duration) = duration {
            std::thread::sleep(duration);
        }
        let value = generate_fn_inner(k);
        if let Some(inner) = count {
            *inner.lock().unwrap() += 1;
        }
        value
    })
}

pub(crate) fn tuple_sum(tuple: &(u64, u64)) -> u64 {
    tuple.0 + tuple.1
}

pub(crate) fn tuple_multiply(tuple: &(u64, u64)) -> u64 {
    tuple.0 * tuple.1
}

/// Generates values corresponding to the same key in two namespaces, potentially multiple times.
///
/// The generate function for each namespace should only be called once, adding 2 to the count of
/// generate function calls (unless the values are already computed before calling this function.
pub(crate) fn run_basic_test(
    root: impl AsRef<Path>,
    client_kind: ClientKind,
    count: Option<Arc<Mutex<u64>>>,
    duration: Option<Duration>,
    handle: &Handle,
) -> Result<()> {
    let root = root.as_ref();

    let (_, local, remote) =
        create_server_and_clients(root.to_path_buf(), client_kind.into(), handle);

    let client = match client_kind {
        ClientKind::Local => local,
        ClientKind::Remote => remote,
    };

    let handle1 = cached_generate(
        &client,
        duration,
        count.clone(),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_GENERATE_FN,
    );
    let handle2 = cached_generate(
        &client,
        duration,
        count.clone(),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_GENERATE_FN,
    );

    assert_eq!(*handle1.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));
    assert_eq!(*handle2.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    let handle1 = cached_generate(
        &client,
        duration,
        count.clone(),
        BASIC_TEST_ALT_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_ALT_GENERATE_FN,
    );
    let handle2 = cached_generate(
        &client,
        duration,
        count,
        BASIC_TEST_ALT_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_ALT_GENERATE_FN,
    );

    assert_eq!(
        *handle1.get(),
        BASIC_TEST_ALT_GENERATE_FN(&BASIC_TEST_PARAM)
    );
    assert_eq!(
        *handle2.get(),
        BASIC_TEST_ALT_GENERATE_FN(&BASIC_TEST_PARAM)
    );

    Ok(())
}

pub(crate) fn run_basic_persistence_test(test_name: &str, client_kind: ClientKind) -> Result<()> {
    let (root, count, runtime) = setup_test(test_name)?;

    run_basic_test(
        &root,
        client_kind,
        Some(count.clone()),
        None,
        runtime.handle(),
    )?;

    runtime.shutdown_timeout(Duration::from_millis(500));
    let runtime = create_runtime();

    let (_, local, remote) =
        create_server_and_clients(root.clone(), client_kind.into(), runtime.handle());

    let client = match client_kind {
        ClientKind::Local => local,
        ClientKind::Remote => remote,
    };

    let handle = cached_generate(
        &client,
        None,
        Some(count.clone()),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_GENERATE_FN,
    );
    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    let handle = cached_generate(
        &client,
        None,
        Some(count.clone()),
        BASIC_TEST_ALT_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_ALT_GENERATE_FN,
    );
    assert_eq!(*handle.get(), BASIC_TEST_ALT_GENERATE_FN(&BASIC_TEST_PARAM));

    assert_eq!(*count.lock().unwrap(), 2);

    Ok(())
}

pub(crate) fn run_basic_long_running_task_test(
    test_name: &str,
    client_kind: ClientKind,
) -> Result<()> {
    let (root, count, runtime) = setup_test(test_name)?;
    run_basic_test(
        root,
        client_kind,
        Some(count.clone()),
        Some(TEST_SERVER_HEARTBEAT_TIMEOUT + Duration::from_millis(500)),
        runtime.handle(),
    )?;
    assert_eq!(*count.lock().unwrap(), 2);
    Ok(())
}

pub(crate) fn run_failure_test(
    test_name: &str,
    client_kind: ClientKind,
    restart_server: bool,
) -> Result<()> {
    let (root, count, mut runtime) = setup_test(test_name)?;

    let (_, local, remote) =
        create_server_and_clients(root.clone(), client_kind.into(), runtime.handle());

    let mut client = match client_kind {
        ClientKind::Local => local,
        ClientKind::Remote => remote,
    };

    // Generator should panic and stop sending heartbeats. Since the generator does not
    // successfully, the task should be reassigned.
    let handle1 = cached_generate(
        &client,
        None,
        None,
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        |_param| -> u64 { panic!() },
    );

    assert!(matches!(handle1.get_err().as_ref(), Error::Panic));

    if restart_server {
        runtime.shutdown_timeout(Duration::from_millis(500));
        runtime = create_runtime();

        let (_, local, remote) =
            create_server_and_clients(root, client_kind.into(), runtime.handle());

        client = match client_kind {
            ClientKind::Local => local,
            ClientKind::Remote => remote,
        };
    }

    // The task should be assigned once, and new requesters should be able to retrieve the new
    // value.
    let handle2 = cached_generate(
        &client,
        None,
        Some(count.clone()),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_GENERATE_FN,
    );
    let handle3 = cached_generate(
        &client,
        None,
        Some(count.clone()),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_GENERATE_FN,
    );

    assert_eq!(*handle2.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));
    assert_eq!(*handle3.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));
    assert_eq!(*count.lock().unwrap(), 1);
    Ok(())
}

pub(crate) fn run_cacheable_api_test(test_name: &str, client_kind: ClientKind) -> Result<()> {
    let (root, _, runtime) = setup_test(test_name)?;

    let (_, local, remote) = create_server_and_clients(root, client_kind.into(), runtime.handle());

    let client = match client_kind {
        ClientKind::Local => local,
        ClientKind::Remote => remote,
    };

    let handle1 = client.get(BASIC_TEST_NAMESPACE, Key(0));
    let handle2 = client.get(BASIC_TEST_NAMESPACE, Key(5));
    let handle3 = client.get(BASIC_TEST_NAMESPACE, Key(8));

    assert_eq!(*handle1.unwrap_inner(), 0);
    assert_eq!(
        format!("{}", handle2.unwrap_err_inner().root_cause()),
        "invalid key"
    );
    assert!(matches!(handle3.get_err().as_ref(), Error::Panic));

    let state = Arc::new(Mutex::new(Vec::new()));
    let handle1 = client.get_with_state(BASIC_TEST_ALT_NAMESPACE, Key(0), state.clone());
    let handle2 = client.get_with_state(BASIC_TEST_ALT_NAMESPACE, Key(5), state.clone());
    let handle3 = client.get_with_state(BASIC_TEST_ALT_NAMESPACE, Key(8), state.clone());

    assert_eq!(*handle1.unwrap_inner(), 0);
    assert_eq!(
        format!("{}", handle2.unwrap_err_inner().root_cause()),
        "invalid key"
    );
    assert!(matches!(handle3.get_err().as_ref(), Error::Panic));

    assert_eq!(state.lock().unwrap().clone(), vec![0]);

    Ok(())
}

#[test]
fn servers_cannot_be_started_with_same_root() -> Result<()> {
    let (root, _, runtime) = setup_test("servers_cannot_be_started_with_same_root")?;
    let (_, _, _) = create_server_and_clients(root.clone(), ServerKind::Local, runtime.handle());
    std::thread::sleep(Duration::from_millis(100));
    let (server2, _, _) = create_server_and_clients(root, ServerKind::Remote, runtime.handle());
    assert!(server2.get().is_err());
    Ok(())
}

#[test]
fn local_server_persists_cached_values() -> Result<()> {
    run_basic_persistence_test("local_server_persists_cached_values", ClientKind::Local)
}

#[test]
fn remote_server_persists_cached_values() -> Result<()> {
    run_basic_persistence_test("remote_server_persists_cached_values", ClientKind::Remote)
}

#[test]
fn local_client_cacheable_api_works() -> Result<()> {
    run_cacheable_api_test("local_client_cacheable_api_works", ClientKind::Local)
}

#[test]
fn remote_client_cacheable_api_works() -> Result<()> {
    run_cacheable_api_test("remote_client_cacheable_api_works", ClientKind::Remote)
}

#[test]
fn local_remote_apis_work_concurrently() -> Result<()> {
    let (root, count, runtime) = setup_test("local_remote_apis_work_concurrently")?;

    let (_, local, remote) =
        create_server_and_clients(root.to_path_buf(), ServerKind::Both, runtime.handle());

    let mut handles = Vec::new();

    for _ in 0..5 {
        for client in [&local, &remote] {
            handles.push(cached_generate(
                client,
                None,
                Some(count.clone()),
                BASIC_TEST_NAMESPACE,
                BASIC_TEST_PARAM,
                BASIC_TEST_GENERATE_FN,
            ));
        }
    }

    for handle in handles {
        assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));
    }

    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}

#[test]
fn local_server_does_not_reassign_long_running_tasks() -> Result<()> {
    run_basic_long_running_task_test(
        "local_server_does_not_reassign_long_running_tasks",
        ClientKind::Local,
    )
}

#[test]
fn remote_server_does_not_reassign_long_running_tasks() -> Result<()> {
    run_basic_long_running_task_test(
        "remote_server_does_not_reassign_long_running_tasks",
        ClientKind::Remote,
    )
}

#[test]
fn local_server_reassigns_failed_tasks() -> Result<()> {
    run_failure_test(
        "local_server_reassigns_failed_tasks",
        ClientKind::Local,
        false,
    )
}

#[test]
fn remote_server_reassigns_failed_tasks() -> Result<()> {
    run_failure_test(
        "remote_server_reassigns_failed_tasks",
        ClientKind::Remote,
        false,
    )
}

#[test]
fn local_server_recovers_from_failures_on_restart() -> Result<()> {
    run_failure_test(
        "local_server_recovers_from_failures_on_restart",
        ClientKind::Local,
        true,
    )
}

#[test]
fn remote_server_recovers_from_failures_on_restart() -> Result<()> {
    run_failure_test(
        "remote_server_recovers_from_failures_on_restart",
        ClientKind::Remote,
        true,
    )
}
