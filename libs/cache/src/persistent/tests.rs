use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    error::Result,
    persistent::{
        client::{ClientConfig, LocalClient, RemoteClient},
        server::{Server, HEARTBEAT_TIMEOUT_SECS_DEFAULT},
    },
};

const BUILD_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/build");

fn start_remote_server(root: PathBuf, port: u16) -> Server {
    let server = Server::builder()
        .root(root)
        .remote(format!("0.0.0.0:{port}").parse().unwrap())
        .build()
}

fn start_local_server(root: PathBuf, port: u16) -> Server {
    let server = Server::builder()
        .root(root)
        .remote(format!("0.0.0.0:{port}").parse().unwrap())
        .build()
}

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

    let server = Server::builder()
        .root(root.clone())
        .remote("0.0.0.0:28055".parse().unwrap())
        .build();
    let handle = runtime.spawn(async move { server.start().await });

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let client = RemoteClient::new(
        ClientConfig::builder()
            .url("https://0.0.0.0:28055".to_string())
            .build(),
    );
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

    let server = Server::builder()
        .root(root.clone())
        .remote("0.0.0.0:28055".parse().unwrap())
        .build();
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

    let server = Server::builder()
        .root(root.clone())
        .local("0.0.0.0:28056".parse().unwrap())
        .build();
    let handle = runtime.spawn(async move { server.start().await });

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let client = LocalClient::new(
        ClientConfig::builder()
            .url("https://0.0.0.0:28056".to_string())
            .build(),
    );
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

    let server = Server::builder()
        .root(root.clone())
        .local("0.0.0.0:28056".parse().unwrap())
        .build();
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

    let server = Server::builder()
        .root(root.clone())
        .remote("0.0.0.0:28057".parse().unwrap())
        .build();
    drop(runtime.spawn(async move { server.start().await }));

    std::thread::sleep(Duration::from_millis(400)); // Wait until server starts.

    let client = RemoteClient::new(
        ClientConfig::builder()
            .url("https://0.0.0.0:28057".to_string())
            .build(),
    );
    let count = Arc::new(Mutex::new(0));
    let count1 = count.clone();
    let count2 = count.clone();
    let handle1 = client.generate("test", (1, 2, 3), move |k| {
        *count1.lock().unwrap() += 1;
        std::thread::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS_DEFAULT + 1));
        k.0 + k.1 + k.2
    });
    let handle2 = client.generate("test", (1, 2, 3), move |k| {
        *count2.lock().unwrap() += 1;
        std::thread::sleep(Duration::from_secs(HEARTBEAT_TIMEOUT_SECS_DEFAULT + 1));
        k.0 + k.1 + k.2
    });

    assert_eq!(handle1.get(), handle2.get());
    assert_eq!(*count.lock().unwrap(), 1);

    Ok(())
}
