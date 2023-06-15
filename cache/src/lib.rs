use client::remote_cache_put;
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha512};

use crate::client::remote_cache_get;

pub mod client;
pub mod rest_api;

pub fn get<T>(id: impl Into<String>, key: impl Serialize, gen_value: impl FnOnce() -> T) -> T
where
    T: Serialize + DeserializeOwned,
{
    let mut hasher = Sha512::new();
    hasher.update(&flexbuffers::to_vec(key).unwrap());
    let key: Vec<u8> = hasher.finalize()[..].into();

    let id: String = id.into();

    if let Some(v) = remote_cache_get("http://0.0.0.0:3000", id.clone(), key.clone()) {
        return flexbuffers::from_slice(&v).unwrap();
    }

    let value = gen_value();

    let ser_value = flexbuffers::to_vec(&value).unwrap();

    remote_cache_put("http://0.0.0.0:3000", id, key, ser_value);

    value
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::get;

    #[test]
    #[ignore = "requires starting cache backend manually"]
    fn cache_computation() {
        let misses = Arc::new(Mutex::new(0));

        let x = get("test", (2, 5), || {
            *misses.lock().unwrap() += 1;
            2 + 5
        });

        assert_eq!(x, 7);

        let y = get("test", (2, 5), || {
            *misses.lock().unwrap() += 1;
            2 + 5
        });

        assert_eq!(x, y);
        assert_eq!(*misses.lock().unwrap(), 1);
    }
}
