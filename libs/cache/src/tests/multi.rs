use std::{sync::mpsc::channel, time::Duration};

use test_log::test;

use crate::{
    error::Result,
    multi::MultiCache,
    persistent::client::{create_server_and_clients, setup_test, ServerKind},
    tests::persistent::{
        cached_generate, BASIC_TEST_GENERATE_FN, BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM,
    },
};

#[test]
fn multi_cache_writes_through() -> Result<()> {
    let (root, count, runtime) = setup_test("multi_cache_writes_through")?;

    let (_, local, _) = create_server_and_clients(root, ServerKind::Local, runtime.handle());

    let mut cache = MultiCache::builder().with_provider(local.clone()).build();

    let count_clone = count.clone();

    let handle = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    let handle = cached_generate(
        &local,
        None,
        Some(count.clone()),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        BASIC_TEST_GENERATE_FN,
    );

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}

#[test]
fn multi_cache_blocks_on_generator_in_nested_cache() -> Result<()> {
    let (root, count, runtime) = setup_test("multi_cache_blocks_on_generator_in_nested_cache")?;

    let (_, local, _) = create_server_and_clients(root, ServerKind::Local, runtime.handle());

    let (s, r) = channel();

    let handle1 = cached_generate(
        &local,
        None,
        Some(count.clone()),
        BASIC_TEST_NAMESPACE,
        BASIC_TEST_PARAM,
        move |key| {
            r.recv().unwrap();
            BASIC_TEST_GENERATE_FN(key)
        },
    );

    let mut cache = MultiCache::builder().with_provider(local).build();

    let count_clone = count.clone();

    let handle2 = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert!(handle1.poll().is_none());
    assert!(handle2.poll().is_none());

    s.send(()).unwrap();

    assert_eq!(*handle1.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));
    assert_eq!(*handle2.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}

#[test]
fn multi_cache_caches_results_in_memory() -> Result<()> {
    let (root, count, runtime) = setup_test("multi_cache_caches_results_in_memory")?;

    let (_, local, _) = create_server_and_clients(root, ServerKind::Local, runtime.handle());

    let mut cache = MultiCache::builder().with_provider(local).build();

    let count_clone = count.clone();

    let handle = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    runtime.shutdown_timeout(Duration::from_millis(1000));

    let count_clone = count.clone();

    let handle = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}

#[test]
fn multi_cache_works_without_in_memory_caching() -> Result<()> {
    let (root, count, runtime) = setup_test("multi_cache_works_without_in_memory_caching")?;

    let (_, local, _) = create_server_and_clients(root, ServerKind::Local, runtime.handle());

    let mut cache = MultiCache::builder()
        .skip_memory()
        .with_provider(local)
        .build();

    let count_clone = count.clone();

    let handle = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    let count_clone = count.clone();

    let handle = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    runtime.shutdown_timeout(Duration::from_millis(1000));

    let count_clone = count.clone();

    let handle = cache.generate(BASIC_TEST_NAMESPACE, BASIC_TEST_PARAM, move |key| {
        *count_clone.lock().unwrap() += 1;
        BASIC_TEST_GENERATE_FN(key)
    });

    assert_eq!(*handle.get(), BASIC_TEST_GENERATE_FN(&BASIC_TEST_PARAM));

    assert_eq!(*count.lock().unwrap(), 2);

    Ok(())
}
