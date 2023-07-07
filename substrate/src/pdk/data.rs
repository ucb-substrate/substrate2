//! PDK-provided data structures.

use std::collections::HashMap;

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};

/// Top-level specification of a PDK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdkSpec {
    pdk: PdkDef,
    layers: LayerFamilies,
}

/// PDK declaration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdkDef {
    name: ArcStr,
}

/// A collection of layer families.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LayerFamilies {
    inner: HashMap<ArcStr, LayerFamily>,
}

/// A single layer family, composed of individual [`Layer`]s.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerFamily {
    routing_kind: RoutingKind,
    #[serde(flatten)]
    elements: HashMap<ArcStr, Layer>,
}

/// A single layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    name: ArcStr,
    gds: Option<(i16, i16)>,
}

/// An enumeration of possible uses of a layer for routing.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingKind {
    /// A low-level layer not directly used for routing.
    ///
    /// Examples: poly, diffusion, high threshold implant.
    #[default]
    Base,
    /// A routing metal layer.
    Routing,
    /// A cut used to connect metal layers.
    Cut,
    /// Indicates a routing obstruction.
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
