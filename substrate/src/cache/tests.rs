use std::sync::{Arc, Mutex};

use cache::Cacheable;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use substrate::io::schematic::HardwareType;
use substrate::schematic::{CellBuilder, HasNestedView, InstancePath};
use substrate::{
    block::Block,
    context::PdkContext,
    schematic::{ExportsNestedData, Schematic},
};

use crate::shared::pdk::ExamplePdkA;

lazy_static! {
    static ref RUNS: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
}

#[derive(Serialize, Deserialize, Hash, Eq, PartialEq, Copy, Clone, Debug)]
pub struct CachedDesignScript(u64);

impl Cacheable for CachedDesignScript {
    type Output = u64;
    type Error = substrate::error::Error;

    fn generate(&self) -> Result<Self::Output, Self::Error> {
        println!("Running design script");
        *RUNS.lock().unwrap() += 1;
        Ok(self.0 * 5)
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct CacheBlock(u64);

impl Block for CacheBlock {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("cacheblock")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("cacheblock")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheBlockData {
    design: u64,
}

impl HasNestedView for CacheBlockData {
    type NestedView = Self;

    fn nested_view(&self, _parent: &InstancePath) -> Self::NestedView {
        *self
    }
}

impl ExportsNestedData for CacheBlock {
    type NestedData = CacheBlockData;
}

impl Schematic<ExamplePdkA> for CacheBlock {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        let design = *cell
            .ctx()
            .cache
            .type_get(CachedDesignScript(5))
            .unwrap_inner();

        Ok(CacheBlockData { design })
    }
}

#[test]
fn caching_works() {
    let ctx = PdkContext::new(ExamplePdkA);
    for i in 0..5 {
        // Generates 5 different blocks that share the same design script.
        //
        // Should only run the design script once even though 5 schematics are generated.
        let handle = ctx.generate_schematic::<ExamplePdkA, _>(CacheBlock(i));
        assert_eq!(handle.cell().design, 25);
    }
    assert_eq!(*RUNS.lock().unwrap(), 1);
}
