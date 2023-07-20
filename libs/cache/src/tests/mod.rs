use std::sync::{Arc, Mutex};

use anyhow::bail;
use serde::{Deserialize, Serialize};

use crate::{Cacheable, CacheableWithState};

pub(crate) mod mem;
pub(crate) mod multi;
pub(crate) mod persistent;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct Key(u64);

impl Cacheable for Key {
    type Output = u64;
    type Error = anyhow::Error;

    fn generate(&self) -> Result<Self::Output, Self::Error> {
        if self.0 == 5 {
            bail!("invalid key");
        } else if self.0 == 8 {
            panic!("generator panicked");
        }
        Ok(self.0)
    }
}

impl CacheableWithState<Arc<Mutex<Vec<u64>>>> for Key {
    type Output = u64;
    type Error = anyhow::Error;

    fn generate_with_state(
        &self,
        state: Arc<Mutex<Vec<u64>>>,
    ) -> Result<Self::Output, Self::Error> {
        let out = self.generate()?;
        println!("generating");
        state.lock().unwrap().push(out);
        Ok(out)
    }
}
