use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::Result;
use cache::persistent::client::ClientKind;

use crate::{raw::RawConfig, CacheProviderConfig, Config};

const DATA_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/data");

#[test]
fn test_raw_config() -> Result<()> {
    let mut cfg = RawConfig::new(
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

    let providers: Option<HashMap<String, CacheProviderConfig>> = cfg.get("cache.providers")?;
    let providers = providers.unwrap();

    assert_eq!(
        providers.get("bwrc"),
        Some(&CacheProviderConfig {
            kind: ClientKind::Local,
            url: "http://0.0.0.0:1234".to_string(),
        })
    );

    assert_eq!(
        providers.get("public"),
        Some(&CacheProviderConfig {
            kind: ClientKind::Remote,
            url: "https://cache.substratelabs.io:1234".to_string(),
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

#[test]
fn test_config() -> Result<()> {
    let cfg = Config::new(
        PathBuf::from(DATA_DIR).join("test_substrate_config/projects"),
        "".into(),
    )?;

    assert!(cfg.cache.enable);
    assert!(cfg.cache.skip_memory);
    assert_eq!(
        cfg.cache.providers.get("bwrc"),
        Some(&CacheProviderConfig {
            kind: ClientKind::Local,
            url: "http://0.0.0.0:1234".to_string(),
        })
    );
    assert_eq!(
        cfg.cache.providers.get("public"),
        Some(&CacheProviderConfig {
            kind: ClientKind::Remote,
            url: "https://cache.substratelabs.io:1234".to_string(),
        })
    );
    assert_eq!(
        cfg.cache.selected_providers,
        HashSet::from_iter(["bwrc".to_string(), "public".to_string()])
    );

    Ok(())
}
