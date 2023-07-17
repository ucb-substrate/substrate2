use cache::{
    mem::{NamespaceCache, TypeCache},
    persistent::client::Client,
};

pub struct Cache {
    type_cache: TypeCache,
    namespace_cache: NamespaceCache,
    persistent_client: Option<Client>,
}

impl Cache {
    fn new(persistent_client: Option<Client>) -> Self {
        Self {}
    }
}
