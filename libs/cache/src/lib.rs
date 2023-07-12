//! Caching utilities.
#![warn(missing_docs)]

use std::{any::Any, fmt::Debug, hash::Hash, sync::Arc, thread};

use error::{ArcResult, Error, Result};
use once_cell::sync::OnceCell;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod error;
pub mod mem;
pub mod persistent;
#[doc(hidden)]
pub mod rpc;

/// A cacheable object.
///
/// # Examples
///
/// ```
/// use cache::Cacheable;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize, Hash, Eq, PartialEq)]
/// pub struct Params {
///     param1: u64,
///     param2: String,
/// };
///
/// impl Cacheable for Params {
///     type Output = u64;
///     type Error = anyhow::Error;
///
///     fn generate(&self) -> anyhow::Result<u64> {
///         println!("Executing an expensive computation...");
///
///         // ...
///         # let error_condition = true;
///         # let computation_result = 64;
///
///         if error_condition {
///             anyhow::bail!("an error occured during computation");
///         }
///
///         Ok(computation_result)
///     }
/// }
/// ```
pub trait Cacheable: Serialize + DeserializeOwned + Hash + Eq + Send + Sync + Any {
    /// The output produced by generating the object.
    type Output: Send + Sync + Serialize + DeserializeOwned;
    /// The error type returned by [`Cacheable::generate`].
    type Error: Send + Sync;

    /// Generates the output of the cacheable object.
    fn generate(&self) -> std::result::Result<Self::Output, Self::Error>;
}

/// A cacheable object whose generator needs to store state.
///
/// # Examples
///
/// ```
/// use std::sync::{Arc, Mutex};
/// use cache::CacheableWithState;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize, Serialize, Clone, Hash, Eq, PartialEq)]
/// pub struct Params {
///     param1: u64,
///     param2: String,
/// };
///
/// #[derive(Clone)]
/// pub struct Log(Arc<Mutex<Vec<Params>>>);
///
/// impl CacheableWithState<Log> for Params {
///     type Output = u64;
///     type Error = anyhow::Error;
///
///     fn generate_with_state(&self, state: Log) -> anyhow::Result<u64> {
///         println!("Logging parameters...");
///         state.0.lock().unwrap().push(self.clone());
///
///         println!("Executing an expensive computation...");
///
///         // ...
///         # let error_condition = true;
///         # let computation_result = 64;
///
///         if error_condition {
///             anyhow::bail!("an error occured during computation");
///         }
///
///         Ok(computation_result)
///     }
/// }
/// ```
pub trait CacheableWithState<S: Send + Sync + Any>:
    Serialize + DeserializeOwned + Hash + Eq + Send + Sync + Any
{
    /// The output produced by generating the object.
    type Output: Send + Sync + Serialize + DeserializeOwned;
    /// The error type returned by [`CacheableWithState::generate_with_state`].
    type Error: Send + Sync;

    /// Generates the output of the cacheable object using `state`.
    ///
    /// **Note:** The state is not used to determine whether the object should be regenerated. As
    /// such, it should not impact the output of this function but rather should only be used to
    /// store collateral or reuse computation from other function calls.
    fn generate_with_state(&self, state: S) -> std::result::Result<Self::Output, Self::Error>;
}

/// A handle to a cache entry that might still be generating.
#[derive(Debug)]
pub struct CacheHandle<V>(pub(crate) Arc<OnceCell<ArcResult<V>>>);

impl<V> Clone for CacheHandle<V> {
    fn clone(&self) -> Self {
        CacheHandle(self.0.clone())
    }
}

impl<V: Send + Sync + Any> CacheHandle<V> {
    pub fn new(generate_fn: impl FnOnce() -> V + Send + Sync + Any) -> Self {
        let handle = Self(Arc::new(OnceCell::new()));

        let handle2 = handle.clone();
        thread::spawn(move || {
            let handle3 = handle2.clone();
            let join_handle = thread::spawn(move || {
                let value = generate_fn();
                if handle3.0.set(Ok(value)).is_err() {
                    panic!("failed to set cell value");
                }
            });
            if join_handle.join().is_err() && handle2.0.set(Err(Arc::new(Error::Panic))).is_err() {
                panic!("failed to set cell value on panic");
            }
        });

        handle
    }
}

impl<V> CacheHandle<V> {
    /// Blocks on the cache entry, returning the result once it is ready.
    ///
    /// Returns an error if one was returned by the generator.
    pub fn try_get(&self) -> ArcResult<&V> {
        self.0.wait().as_ref().map_err(|e| e.clone())
    }

    /// Checks whether the underlying entry is ready.
    ///
    /// Returns the entry if available, otherwise returns [`None`].
    pub fn poll(&self) -> Option<&ArcResult<V>> {
        self.0.get()
    }

    /// Blocks on the cache entry, returning its output.
    ///
    /// # Panics
    ///
    /// Panics if the generator failed to run or an internal error was thrown by the cache.
    pub fn get(&self) -> &V {
        self.try_get().unwrap()
    }
}

impl<V: Debug> CacheHandle<V> {
    /// Blocks on the cache entry, returning the error thrown by the cache.
    ///
    /// # Panics
    ///
    /// Panics if no error was thrown by the cache.
    pub fn get_err(&self) -> Arc<error::Error> {
        self.try_get().unwrap_err()
    }
}

/// The error type returned by [`CacheHandle::try_inner`].
pub enum TryInnerError<'a, E> {
    /// An error thrown by the cache.
    CacheError(Arc<error::Error>),
    /// An error thrown by the generator.
    GeneratorError(&'a E),
}

impl<'a, E> From<&'a E> for TryInnerError<'a, E> {
    fn from(value: &'a E) -> Self {
        Self::GeneratorError(value)
    }
}

impl<V, E> CacheHandle<std::result::Result<V, E>> {
    /// Blocks on the cache entry, returning the inner result.
    ///
    /// Returns an error if the generator panicked or threw an error, or if the cache threw an
    /// error.
    pub fn try_inner(&self) -> std::result::Result<&V, TryInnerError<E>> {
        Ok(self
            .try_get()
            .map_err(|e| TryInnerError::CacheError(e))?
            .as_ref()?)
    }
}

impl<V, E: Debug> CacheHandle<std::result::Result<V, E>> {
    /// Blocks on the cache entry, returning its output.
    ///
    /// # Panics
    ///
    /// Panics if the generator panicked or threw an error, or if an internal error was thrown by the cache.
    pub fn unwrap_inner(&self) -> &V {
        self.get().as_ref().unwrap()
    }
}

impl<V: Debug, E> CacheHandle<std::result::Result<V, E>> {
    /// Blocks on the cache entry, returning the error returned by the generator.
    ///
    /// # Panics
    ///
    /// Panics if the generator panicked or an internal error was thrown by the cache. Also panics
    /// if the generator did not return an error.
    pub fn unwrap_err_inner(&self) -> &E {
        self.get().as_ref().unwrap_err()
    }
}

pub(crate) fn hash(val: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(val);
    hasher.finalize()[..].into()
}
