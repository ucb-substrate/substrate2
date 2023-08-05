# Remote Cache

A general purpose cache with possibly multiple remote servers for storing and retrieving data.

The cache includes both type-mapped and namespaced APIs. Caching can be done in-memory or persistently
via a cache server that manages a filesystem cache. The cache also supports caching across
several cache servers.

# Usage

```rust
use cache::{persistent::client::{Client, ClientKind}, error::Error, Cacheable};

let client = Client::with_default_config(ClientKind::Local, "http://127.0.0.1:28055");

fn generate_fn(tuple: &(u64, u64)) -> u64 {
    tuple.0 + tuple.1
}

let handle = client.generate("example.namespace", (5, 6), generate_fn);
assert_eq!(*handle.get(), 11);
```
