use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    error::Result,
    persistent::{
        client::LocalCacheClient,
        server::{CacheServer, HEARTBEAT_TIMEOUT_SECS},
    },
};

use super::RemoteCacheClient;

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

#[test]
fn remote_client_retrieves_persistently_cached_values() -> Result<()> {
    let root = PathBuf::from(BUILD_DIR).join("remote_client_retrieves_persistently_cached_values");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let server = CacheServer::new(root.clone()).with_remote("0.0.0.0:28055".parse().unwrap());
    let handle = runtime.spawn(async move { server.start().await });

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let client = RemoteCacheClient::new(root.clone());
    let count = Arc::new(Mutex::new(0));
    let count1 = count.clone();
    let count2 = count.clone();
    let handle1 = client.generate("test", (1, 2, 3), move |k| {
        *count1.lock().unwrap() += 1;
        k.0 + k.1 + k.2
    });
    let handle2 = client.generate("test", (1, 2, 3), move |k| {
        *count2.lock().unwrap() += 1;
        k.0 + k.1 + k.2
    });

    assert_eq!(handle1.get(), handle2.get());

    handle.abort();

    let server = CacheServer::new(root).with_remote("0.0.0.0:28056".parse().unwrap());
    drop(runtime.spawn(async move { server.start().await }));

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let count2 = count.clone();
    let handle2 = client.generate("test", (1, 2, 3), move |k| {
        *count2.lock().unwrap() += 1;
        k.0 + k.1 + k.2
    });

    assert_eq!(handle1.get(), handle2.get());
    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}

#[test]
fn local_client_retrieves_persistently_cached_values() -> Result<()> {
    let root = PathBuf::from(BUILD_DIR).join("local_client_retrieves_persistently_cached_values");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let server = CacheServer::new(root.clone()).with_local("0.0.0.0:28057".parse().unwrap());
    let handle = runtime.spawn(async move { server.start().await });

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let client = LocalCacheClient::new(root.clone());
    let count = Arc::new(Mutex::new(0));
    let count1 = count.clone();
    let count2 = count.clone();
    let handle1 = client.generate("test", (1, 2, 3), move |k| {
        *count1.lock().unwrap() += 1;
        k.0 + k.1 + k.2
    });
    let handle2 = client.generate("test", (1, 2, 3), move |k| {
        *count2.lock().unwrap() += 1;
        k.0 + k.1 + k.2
    });

    assert_eq!(handle1.get(), handle2.get());

    handle.abort();

    let server = CacheServer::new(root).with_local("0.0.0.0:28058".parse().unwrap());
    drop(runtime.spawn(async move { server.start().await }));

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let count2 = count.clone();
    let handle2 = client.generate("test", (1, 2, 3), move |k| {
        *count2.lock().unwrap() += 1;
        k.0 + k.1 + k.2
    });

    assert_eq!(handle1.get(), handle2.get());
    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}

#[test]
#[ignore = "long"]
fn remote_client_runs_long_running_tasks_once() -> Result<()> {
    let root = PathBuf::from(BUILD_DIR).join("remote_client_runs_long_running_tasks_once");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let server = CacheServer::new(root.clone()).with_remote("0.0.0.0:28059".parse().unwrap());
    drop(runtime.spawn(async move { server.start().await }));

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let client = RemoteCacheClient::new(root);
    let count = Arc::new(Mutex::new(0));
    let count1 = count.clone();
    let count2 = count.clone();
    let handle1 = client.generate("test", (1, 2, 3), move |k| {
        *count1.lock().unwrap() += 1;
        std::thread::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS + 1));
        k.0 + k.1 + k.2
    });
    let handle2 = client.generate("test", (1, 2, 3), move |k| {
        *count2.lock().unwrap() += 1;
        std::thread::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS + 1));
        k.0 + k.1 + k.2
    });

    assert_eq!(handle1.get(), handle2.get());
    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}
