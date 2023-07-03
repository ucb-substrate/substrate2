use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::context::Context;
use substrate::geometry::prelude::*;
use substrate::io::{
    CustomLayoutType, InOut, Input, LayoutPort, Node, Output, PortGeometry, ShapePort, Signal,
};
use substrate::layout::{
    draw::DrawContainer, element::Shape, Cell, HasLayout, HasLayoutImpl, Instance,
};
use substrate::pdk::layers::LayerId;
use substrate::pdk::{Pdk, PdkLayers};
use substrate::supported_pdks;
use substrate::{
    DerivedLayerFamily, DerivedLayers, Io, Layer, LayerFamily, Layers, LayoutData, LayoutType,
};