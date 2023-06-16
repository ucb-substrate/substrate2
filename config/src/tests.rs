use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

use crate::config::Config;

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/data");

#[derive(Debug, Clone, Copy, Eq, PartialEq, Deserialize)]
enum CacheProviderMethod {
    Http,
    Filesystem,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
struct CacheProvider {
    method: CacheProviderMethod,
    addr: String,
    port: usize,
}

#[test]
fn test_substrate_config() -> Result<()> {
    let mut cfg = Config::new(
        PathBuf::from(DATA_DIR).join("test_substrate_config/projects"),
        "".into(),
    );

    cfg.set_env(HashMap::from_iter(
        [
            ("SUBSTRATE_TEST", "54321"),
            ("SUBSTRATE_CACHE_ENABLE", "false"),
        ]
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string())),
    ));

    let providers: Option<HashMap<String, CacheProvider>> = cfg.get("cache.providers")?;
    let providers = providers.unwrap();

    assert_eq!(
        providers.get("bwrc"),
        Some(&CacheProvider {
            method: CacheProviderMethod::Http,
            addr: "substratecache.eecs.berkeley.edu".to_string(),
            port: 1234,
        })
    );

    assert_eq!(
        providers.get("public"),
        Some(&CacheProvider {
            method: CacheProviderMethod::Http,
            addr: "cache.substratelabs.io".to_string(),
            port: 1234,
        })
    );

    // Test environment variable configuration.
    let test: Option<usize> = cfg.get("test")?;
    assert_eq!(test.unwrap(), 54321);

    let cache_enable: Option<bool> = cfg.get("cache.enable")?;
    assert!(!cache_enable.unwrap());

    cfg.set_env(HashMap::new());
    let test: Option<usize> = cfg.get("test")?;
    assert_eq!(test, None);

    let cache_enable: Option<bool> = cfg.get("cache.enable")?;
    assert!(cache_enable.unwrap());

    Ok(())
}
