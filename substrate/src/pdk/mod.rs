//! Traits and utilities for defining process design kits (PDKs).

use std::any::Any;

/// A process development kit.
pub trait Pdk: Send + Sync + Any {}
