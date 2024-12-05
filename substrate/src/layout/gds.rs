//! Utilities for GDS conversion.
//!
//! Converts between Substrate's layout data-model and [`gds`] structures.

use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;
use gds::{GdsUnits, HasLayer};
use geometry::prelude::Polygon;
use geometry::span::Span;
use geometry::transform::{Rotation, Transformation};
use geometry::{
    prelude::{Corner, Orientation, Point},
    rect::Rect,
};
use indexmap::IndexMap;
use layir::Shape;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use slotmap::{new_key_type, SlotMap};
use tracing::{span, Level};
use uniquify::Names;

use crate::io::layout::{BundleBuilder, HardwareType, PortGeometry};
use crate::layout::error::GdsExportError;
use crate::pdk::layers::LayerInfo;
use crate::{
    io::NameBuf,
    pdk::layers::{GdsLayerSpec, HasPin, LayerContext, LayerId},
};

use super::error::{GdsImportError, GdsImportResult};
use super::LayoutContext;
use super::{
    element::{CellId, Element, RawCell, RawInstance},
    error::GdsExportResult,
};
