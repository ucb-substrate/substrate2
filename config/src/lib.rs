//! Substrate's configuration system.
#![warn(missing_docs)]

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use cache::{
    multi::MultiCache,
    persistent::client::{Client, ClientKind},
};
use serde::{Deserialize, Serialize};

use crate::raw::RawConfig;

pub(crate) mod home;
pub(crate) mod paths;
pub(crate) mod raw;
#[cfg(test)]
mod tests;

/// A Substrate configuration instance.
pub struct Config {
    /// Configuration for Substrate's persistent cache.
    pub cache: CacheConfig,
}

impl Config {
    /// Creates a new [`Config`] instance.
    ///
    /// This is typically used for tests or other special cases. [`Config::default`] is
    /// preferred otherwise.
    pub fn new(cwd: PathBuf, homedir: PathBuf) -> Result<Self> {
        Self::from_raw_config(&RawConfig::new(cwd, homedir))
    }

    /// Creates a new [`Config`] instance, with all default settings.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self> {
        Self::from_raw_config(&RawConfig::default()?)
    }

    fn from_raw_config(raw: &RawConfig) -> Result<Self> {
        Ok(Config {
            cache: CacheConfig::from_raw_config(raw)?,
        })
    }
}

/// Configuration for a cache provider.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheProviderConfig {
    /// The API that the cache provider exposes.
    pub kind: ClientKind,
    /// The URL of the cache provider.
    pub url: String,
}

/// Configuration for Substrate's persistent cache.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Set to `true` if the cache is enabled.
    pub enable: bool,
    /// Set to `true` if the cache does not cache results in memory.
    pub skip_memory: bool,
    /// A map of defined providers.
    pub providers: HashMap<String, CacheProviderConfig>,
    /// A list of active providers used by the current project.
    pub selected_providers: HashSet<String>,
}

impl CacheConfig {
    fn from_raw_config(raw: &RawConfig) -> Result<Self> {
        let enable: Option<_> = raw.get("cache.enable")?;
        let skip_memory: Option<_> = raw.get("cache.skip_memory")?;
        let providers: Option<_> = raw.get("cache.providers")?;
        let selected_providers: Option<_> = raw.get("cache.selected_providers")?;

        Ok(Self {
            enable: enable.unwrap_or_default(),
            skip_memory: skip_memory.unwrap_or_default(),
            providers: providers.unwrap_or_default(),
            selected_providers: selected_providers.unwrap_or_default(),
        })
    }

    /// Configures a [`MultiCache`] from the configured options.
    pub fn into_cache(mut self) -> Result<MultiCache> {
        let mut builder = MultiCache::builder();

        if self.skip_memory || !self.enable {
            builder.skip_memory();
        }

        if self.enable {
            for provider in self.selected_providers.iter() {
                let CacheProviderConfig { kind, url } = self
                    .providers
                    .remove(provider)
                    .ok_or_else(|| anyhow!("invalid provider selected"))?;
                builder.with_provider(Client::builder().kind(kind).url(url).build());
            }
        }

        Ok(builder.build())
    }
}
