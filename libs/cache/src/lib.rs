//! A general purpose cache with possibly multiple remote servers for storing and retrieving data.
//!
//! The cache includes both type-mapped and namespaced APIs. Caching can be done in-memory or persistently
//! via a cache server that manages a filesystem cache. The cache also supports caching across
//! several cache servers.
#![warn(missing_docs)]

use std::fmt::Formatter;
use std::ops::Deref;
use std::{any::Any, fmt::Debug, hash::Hash, sync::Arc, thread};

use error::{ArcResult, Error, TryInnerError};
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod error;
pub mod mem;
pub mod multi;
pub mod persistent;
#[doc(hidden)]
pub mod rpc;
#[cfg(test)]
pub(crate) mod tests;

lazy_static! {
    /// A regex for matching valid namespaces.
    pub static ref NAMESPACE_REGEX: Regex =
        Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*\.)*[A-Za-z_][A-Za-z0-9_]*$").unwrap();
}

/// A function for mapping a cached value to another value.
pub trait ValueMapFn<V1, V2>: Fn(ArcResult<&V1>) -> ArcResult<V2> + Send + Sync + Any {}
impl<V1, V2, T: Fn(ArcResult<&V1>) -> ArcResult<V2> + Send + Sync + Any> ValueMapFn<V1, V2> for T {}

/// A function that can be used to generate a value in a background thread.
pub trait RawGenerateFn<V>: FnOnce() -> V + Send + Any {}
impl<V, T: FnOnce() -> V + Send + Any> RawGenerateFn<V> for T {}

/// A function that can be used to generate a value based on a key in a background thread.
pub trait GenerateFn<K, V>: FnOnce(&K) -> V + Send + Any {}
impl<K, V, T: FnOnce(&K) -> V + Send + Any> GenerateFn<K, V> for T {}

/// A stateful function that can be used to generate a value based on a key in a background thread.
pub trait GenerateWithStateFn<K, S, V>: FnOnce(&K, S) -> V + Send + Any {}
impl<K, S, V, T: FnOnce(&K, S) -> V + Send + Any> GenerateWithStateFn<K, S, V> for T {}

/// A function that can be used to generate a result based on a key in a background thread.
pub trait GenerateResultFn<K, V, E>: FnOnce(&K) -> Result<V, E> + Send + Any {}
impl<K, V, E, T: FnOnce(&K) -> Result<V, E> + Send + Any> GenerateResultFn<K, V, E> for T {}

/// A stateful function that can be used to generate a result based on a key in a background thread.
pub trait GenerateResultWithStateFn<K, S, V, E>:
    FnOnce(&K, S) -> Result<V, E> + Send + Any
{
}
impl<K, S, V, E, T: FnOnce(&K, S) -> Result<V, E> + Send + Any>
    GenerateResultWithStateFn<K, S, V, E> for T
{
}

/// A namespace used for addressing a set of cached items.
///
/// Must match the [`NAMESPACE_REGEX`](static@NAMESPACE_REGEX) regular expression.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Namespace(String);

impl Namespace {
    /// Creates a new [`Namespace`].
    ///
    /// # Panics
    ///
    /// Panics if the provided string does not match [`NAMESPACE_REGEX`](static@NAMESPACE_REGEX).
    pub fn new(namespace: impl Into<String>) -> Self {
        let namespace: String = namespace.into();
        if !Namespace::validate(&namespace) {
            panic!(
                "invalid namespace, does not match regex {:?}",
                NAMESPACE_REGEX.as_str(),
            );
        }
        Self(namespace)
    }

    /// Returns `true` if the provided string is a valid namespace.
    pub fn validate(namespace: &str) -> bool {
        NAMESPACE_REGEX.is_match(namespace)
    }

    /// Converts the namespace into its string value.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl<T: Into<String>> From<T> for Namespace {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl Deref for Namespace {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Namespace {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

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

impl<T: Cacheable> Cacheable for Arc<T> {
    type Output = T::Output;
    type Error = T::Error;

    fn generate(&self) -> std::result::Result<Self::Output, Self::Error> {
        <T as Cacheable>::generate(self)
    }
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

impl<S: Send + Sync + Any, T: CacheableWithState<S>> CacheableWithState<S> for Arc<T> {
    type Output = T::Output;
    type Error = T::Error;

    fn generate_with_state(&self, state: S) -> std::result::Result<Self::Output, Self::Error> {
        <T as CacheableWithState<S>>::generate_with_state(self, state)
    }
}

trait CacheValueHolder<V>: Send + Sync {
    /// Blocks on the cache entry, returning the result once it is ready.
    ///
    /// Returns an error if one was returned by the generator.
    fn try_get(&self) -> ArcResult<&V>;

    /// Checks whether the underlying entry is ready.
    ///
    /// Returns the entry if available, otherwise returns [`None`].
    fn poll(&self) -> Option<ArcResult<&V>>;
}

#[derive(Debug)]
pub(crate) struct CacheHandleInner<V>(Arc<OnceCell<ArcResult<V>>>);

impl<V> Default for CacheHandleInner<V> {
    fn default() -> Self {
        Self(Arc::new(OnceCell::new()))
    }
}

impl<V> Clone for CacheHandleInner<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: Send + Sync> CacheValueHolder<V> for CacheHandleInner<V> {
    fn try_get(&self) -> ArcResult<&V> {
        self.0.wait().as_ref().map_err(|e| e.clone())
    }

    fn poll(&self) -> Option<ArcResult<&V>> {
        Some(self.0.get()?.as_ref().map_err(|e| e.clone()))
    }
}

impl<V> CacheHandleInner<V> {
    /// Sets the value of the cache handle.
    ///
    /// # Panics
    ///
    /// Panics if the cache handle has already been set.
    pub(crate) fn set(&self, value: ArcResult<V>) {
        if self.0.set(value).is_err() {
            tracing::error!("failed to set cache handle value");
            panic!("failed to set cache handle value");
        }
    }
}

/// A handle to a cache entry that might still be generating.
pub struct CacheHandle<V>(Arc<dyn CacheValueHolder<V>>);

impl<V> std::fmt::Debug for CacheHandle<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheHandle").finish()
    }
}

impl<V> Clone for CacheHandle<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl<V: Any + Send + Sync> CacheHandle<V> {
    pub(crate) fn from_inner(inner: Arc<dyn CacheValueHolder<V>>) -> Self {
        Self(inner)
    }

    pub(crate) fn empty() -> (Self, CacheHandleInner<V>) {
        let inner = CacheHandleInner::default();
        (Self(Arc::new(inner.clone())), inner)
    }

    /// Creates a new cache handle, generating its value immediately.
    pub(crate) fn new_blocking(generate_fn: impl RawGenerateFn<V>) -> Self {
        let (handle, inner) = Self::empty();
        inner.set(run_generator(generate_fn));
        handle
    }

    /// Creates a new cache handle, spawning a thread to generate its value using the provided
    /// function.
    pub(crate) fn new(generate_fn: impl RawGenerateFn<V>) -> Self {
        let (handle, inner) = Self::empty();
        thread::spawn(move || {
            inner.set(run_generator(generate_fn));
        });
        handle
    }

    /// Maps an existing [`CacheHandle`] to a [`CacheHandle`] of a different type.
    ///
    /// # Example
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    /// use cache::{mem::TypeCache, error::Error, CacheableWithState};
    ///
    /// let mut cache = TypeCache::new();
    ///
    /// fn generate_fn(tuple: &(u64, u64)) -> u64 {
    ///     tuple.0 + tuple.1
    /// }
    ///
    /// let handle = cache.generate((5, 6), generate_fn);
    /// assert_eq!(*handle.get(), 11);
    ///
    /// // Does not call `generate_fn` again as the result has been cached.
    /// let mapped_handle = handle.map(|res| res.map(|sum| *sum > 50));
    /// assert_eq!(*mapped_handle.get(), false);
    /// ```
    pub fn map<V2: Send + Sync + Any>(&self, map_fn: impl ValueMapFn<V, V2>) -> CacheHandle<V2> {
        CacheHandle(Arc::new(MappedCacheHandleInner::new(self.clone(), map_fn)))
    }
}

impl<V> CacheHandle<V> {
    /// Blocks on the cache entry, returning the result once it is ready.
    ///
    /// Returns an error if one was returned by the generator.
    pub fn try_get(&self) -> ArcResult<&V> {
        self.0.try_get()
    }

    /// Checks whether the underlying entry is ready.
    ///
    /// Returns the entry if available, otherwise returns [`None`].
    pub fn poll(&self) -> Option<ArcResult<&V>> {
        self.0.poll()
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

pub(crate) struct MappedCacheHandleInner<V1, V2> {
    handle: Arc<dyn CacheValueHolder<V1>>,
    map_fn: Arc<dyn ValueMapFn<V1, V2, Output = ArcResult<V2>>>,
    result: Arc<OnceCell<ArcResult<V2>>>,
}

impl<V1, V2> Clone for MappedCacheHandleInner<V1, V2> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            map_fn: self.map_fn.clone(),
            result: self.result.clone(),
        }
    }
}

impl<V1: Send + Sync, V2: Send + Sync> CacheValueHolder<V2> for MappedCacheHandleInner<V1, V2> {
    fn try_get(&self) -> ArcResult<&V2> {
        self.result
            .get_or_init(|| (self.map_fn)(self.handle.try_get()))
            .as_ref()
            .map_err(|e| e.clone())
    }

    fn poll(&self) -> Option<ArcResult<&V2>> {
        let res = self.handle.poll().map(|res| (self.map_fn)(res))?;
        Some(
            self.result
                .get_or_init(|| res)
                .as_ref()
                .map_err(|e| e.clone()),
        )
    }
}

impl<V1, V2> MappedCacheHandleInner<V1, V2> {
    fn new(handle: CacheHandle<V1>, map_fn: impl ValueMapFn<V1, V2>) -> Self {
        Self {
            handle: handle.0,
            map_fn: Arc::new(map_fn),
            result: Arc::new(OnceCell::new()),
        }
    }
}

pub(crate) fn hash(val: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(val);
    hasher.finalize()[..].into()
}

/// Runs the provided generator in a new thread, returning the result.
pub(crate) fn run_generator<V: Any + Send + Sync>(
    generate_fn: impl FnOnce() -> V + Send + Any,
) -> ArcResult<V> {
    let join_handle = thread::spawn(generate_fn);
    join_handle.join().map_err(|_| Arc::new(Error::Panic))
}
