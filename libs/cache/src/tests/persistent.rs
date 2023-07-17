use std::{
    any::Any,
    fs,
    net::TcpListener,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{de::DeserializeOwned, Serialize};
use test_log::test;
use tokio::{
    runtime::{Handle, Runtime},
    task::JoinHandle,
};

use crate::{
    error::{Error, Result},
    persistent::server::{Server, HEARTBEAT_TIMEOUT_SECS_DEFAULT},
    tests::Key,
    CacheHandle,
};

use crate::persistent::client::{Client, ClientKind};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");
const BASIC_TEST_NAMESPACE: &str = "test";
const BASIC_TEST_PARAM: (u64, u64) = (3, 5);
const BASIC_TEST_GENERATE_FN: fn(&(u64, u64)) -> u64 = tuple_sum;
const BASIC_TEST_ALT_NAMESPACE: &str = "test_alt";
const BASIC_TEST_ALT_GENERATE_FN: fn(&(u64, u64)) -> u64 = tuple_multiply;

fn start_local_server(root: PathBuf, port: u16) -> Server {
    Server::builder()
        .root(root)
        .local(format!("0.0.0.0:{port}").parse().unwrap())
        .build()
}

fn start_remote_server(root: PathBuf, port: u16) -> Server {
    Server::builder()
        .root(root)
        .remote(format!("0.0.0.0:{port}").parse().unwrap())
        .build()
}

fn start_local_remote_server(root: PathBuf, local: u16, remote: u16) -> Server {
    Server::builder()
        .root(root)
        .local(format!("0.0.0.0:{local}").parse().unwrap())
        .remote(format!("0.0.0.0:{remote}").parse().unwrap())
        .build()
}

fn pick_n_ports(n: usize) -> Vec<u16> {
    let mut ports = Vec::new();
    let mut temporary_listeners = Vec::new();

    for _ in 0..n {
        let port = portpicker::pick_unused_port().expect("no ports free");
        temporary_listeners.push(TcpListener::bind(format!("0.0.0.0:{port}")));
        ports.push(port);
    }

    ports
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ServerKind {
    Local,
    Remote,
    Both,
}

impl From<ClientKind> for ServerKind {
    fn from(value: ClientKind) -> Self {
        match value {
            ClientKind::Local => ServerKind::Local,
            ClientKind::Remote => ServerKind::Remote,
        }
    }
}

fn remote_url(port: u16) -> String {
    format!("http://0.0.0.0:{port}")
}

fn create_server_and_clients(
    root: PathBuf,
    kind: ServerKind,
    handle: &Handle,
) -> (JoinHandle<Result<()>>, Client, Client) {
    let ports = pick_n_ports(2);
    (
        {
            let server = match kind {
                ServerKind::Local => start_local_server(root, ports[0]),
                ServerKind::Remote => start_remote_server(root, ports[1]),
                ServerKind::Both => start_local_remote_server(root, ports[0], ports[1]),
            };
            let join_handle = handle.spawn(async move { server.start().await });
            std::thread::sleep(Duration::from_millis(100)); // Wait until server starts.
            join_handle
        },
        Client::with_default_config(ClientKind::Local, remote_url(ports[0])),
        Client::with_default_config(ClientKind::Remote, remote_url(ports[1])),
    )
}

fn reset_directory(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn cached_generate<
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

fn tuple_sum(tuple: &(u64, u64)) -> u64 {
    tuple.0 + tuple.1
}

fn tuple_multiply(tuple: &(u64, u64)) -> u64 {
    tuple.0 * tuple.1
}

fn setup_test(test_name: &str) -> (PathBuf, Arc<Mutex<u64>>, Runtime) {
    (
        PathBuf::from(BUILD_DIR).join(test_name),
        Arc::new(Mutex::new(0)),
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap(),
    )
}

/// Generates values corresponding to the same key in two namespaces, potentially multiple times.
///
/// The generate function for each namespace should only be called once, adding 2 to the count of
/// generate function calls (unless the values are already computed before calling this function.
fn run_basic_test(
    root: impl AsRef<Path>,
    client_kind: ClientKind,
    count: Option<Arc<Mutex<u64>>>,
    duration: Option<Duration>,
    handle: &Handle,
) -> Result<()> {
    let root = root.as_ref();

    reset_directory(root)?;

    let (server, local, remote) =
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

    server.abort();
    Ok(())
}

fn run_basic_persistence_test(test_name: &str, client_kind: ClientKind) -> Result<()> {
    let (root, count, runtime) = setup_test(test_name);

    run_basic_test(
        &root,
        client_kind,
        Some(count.clone()),
        None,
        runtime.handle(),
    )?;

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

fn run_basic_long_running_task_test(test_name: &str, client_kind: ClientKind) -> Result<()> {
    let (root, count, runtime) = setup_test(test_name);
    run_basic_test(
        root,
        client_kind,
        Some(count.clone()),
        Some(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS_DEFAULT + 1)),
        runtime.handle(),
    )?;
    assert_eq!(*count.lock().unwrap(), 2);
    Ok(())
}

fn run_cacheable_api_test(test_name: &str, client_kind: ClientKind) -> Result<()> {
    let (root, _, runtime) = setup_test(test_name);

    reset_directory(&root)?;

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
fn servers_cannot_be_started_with_same_root() {
    let (root, _, runtime) = setup_test("servers_cannot_be_started_with_same_root");
    let (_, _, _) = create_server_and_clients(root.clone(), ServerKind::Local, runtime.handle());
    let (server2, _, _) = create_server_and_clients(root, ServerKind::Remote, runtime.handle());
    assert!(runtime.block_on(server2).unwrap().is_err());
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
    let (root, count, runtime) = setup_test("local_remote_apis_work_concurrently");

    reset_directory(&root)?;

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
#[ignore = "long"]
fn local_server_does_not_reassign_long_running_tasks() -> Result<()> {
    run_basic_long_running_task_test(
        "local_server_does_not_reassign_long_running_tasks",
        ClientKind::Local,
    )
}

#[test]
#[ignore = "long"]
fn remote_server_does_not_reassign_long_running_tasks() -> Result<()> {
    run_basic_long_running_task_test(
        "remote_server_does_not_reassign_long_running_tasks",
        ClientKind::Remote,
    )
}
