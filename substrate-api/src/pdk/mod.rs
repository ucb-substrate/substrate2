use std::collections::HashMap;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdkSpec {
    pdk: PdkDef,
    layers: LayerFamilies,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdkDef {
    name: ArcStr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayerFamilies {
    inner: HashMap<ArcStr, LayerFamily>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerFamily {
    routing_kind: RoutingKind,
    #[serde(flatten)]
    elements: HashMap<ArcStr, Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    name: ArcStr,
    gds: Option<(i16, i16)>,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingKind {
    #[default]
    Base,
    Routing,
    Cut,
    Obstruction,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_example_pdk_toml() {
        let spec = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/data/example_pdk/example_pdk.toml"
        );
        println!("{}", spec);
        let spec = std::fs::read_to_string(spec).unwrap();
        let spec: PdkSpec = toml::from_str(&spec).unwrap();
        assert_eq!(spec.pdk.name, "example_pdk");
        assert_eq!(spec.layers.inner["met1"].routing_kind, RoutingKind::Routing);
        assert_eq!(
            spec.layers.inner["met1"].elements["drawing"].gds,
            Some((1, 2))
        );
        assert_eq!(
            spec.layers.inner["met1"].elements["drawing"].name,
            "met1_drawing",
        );
        assert_eq!(spec.layers.inner["via1"].routing_kind, RoutingKind::Cut,);
    }
}
