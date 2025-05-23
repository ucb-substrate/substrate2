//! A library for parsing and writing GDSII files.
//!
//! GDSII is the IC industry's de facto standard for storing and sharing layout data.
//! `gds` is a crate for reading and creating GDSII data based on [Gds21](https://github.com/dan-fritchman/Layout21/tree/main/gds21) and similar to libraries such as [gdstk](https://github.com/heitzmann/gdstk) and its predecessor [gdspy](https://github.com/heitzmann/gdspy).
//! Reading and generating GDSII-format data are primary goals;
//! offering ease-of-use functionality for more elaborate manipulations of GDS data is not.
//!
//! Layout data is represented in three primary forms:
//!
//! * A short tree with three layers:
//!   * The root is a [`GdsLibrary`], which primarily consists of a set of cells ([`GdsStruct`]s), and secondarily a set of metadata.
//!     Each [`GdsLibrary`] is a universe unto itself, in that it has no mechanisms for comprehending layout cells or data defined outside itself.
//!     On-disk each [`GdsLibrary`] is typically paired one-to-one with a `.gds` file.
//!   * Libraries consist of cell definitions AKA [`GdsStruct`]s, which define each layout cell (or module, or "struct" in GDSII terms).
//!   * Cells consist of [`GdsElement`]s, an enumeration which includes individual polygons ([`GdsBoundary`]),
//!     instances of other layout cells ([`GdsStructRef`]), text ([`GdsTextElem`]), and a few other geometric elements.
//! * For storage on disk, the [`GdsLibrary`] tree is flattened to a series of [`GdsRecord`]s.
//!   These records indicate the beginning, end, and content of each tree-node.
//!   Detailed descriptions of these records comprise the majority of the GDSII spec.
//! * Records are stored on-disk in binary form as detailed in the GDSII spec.
//!   Each includes a record-type header, datatype, length field, and optional additional content.
//!   These raw-bytes are never stored by Gds21, only generated and consumed on their way into and out of [`Read`] and [`Write`] objects (typically [`File`]s).
//!
//!
//! ### Usage
//!
//! Loading a [`GdsLibrary`] from disk:
//!
//! ```skip
//! let lib = GdsLibrary::load("sample.gds")?;
//! ```
//!
//! Creating a new and empty [`GdsLibrary`], and adding a [`GdsStruct`] cell-definition:
//!
//! ```rust
//! use gds::{GdsLibrary, GdsStruct};
//! let mut lib = GdsLibrary::new("mylib");
//! lib.structs.push(GdsStruct::new("mycell"));
//! ```
//!
//! Saving a [`GdsLibrary`] to disk:
//!
//! ```skip
//! lib.save("mylib.gds");
//! ```
//!
//! ### Serialization
//!
//! Each element in Gds21's [`GdsLibrary`] tree is [`serde`]-serializable.
//! GDSII data can be straightforwardly serialized in any serde-supported format.
//! Examples:
//!
//! ```
//! let lib = gds::GdsLibrary::new("mylib");
//! let json = serde_json::to_string(&lib);
//! let yaml = serde_yaml::to_string(&lib);
//! let toml = toml::to_string(&lib);
//! ```
//!
//! Gds21 includes built-in support for a subset of serde-formats via its [`SerializationFormat`] enumeration,
//! and support for directly reading and writing files in each format via its accompanying [`SerdeFile`] trait.
//! Example using [`SerializationFormat::Yaml`]:
//!
//! ```skip
//! use gds::SerializationFormat::Yaml;
//! let lib = gds::GdsLibrary::new("mylib");
//!
//! // Write to YAML-format file
//! Yaml.save(&lib, "mylib.gds.yaml")?;
//! // And read back from file
//! let lib2: gds::GdsLibrary = Yaml.open("mylib.gds.yaml")?;
//! ```
//!
//! Note these text-based representations will generally be substantially larger than binary GDSII data.
#![warn(missing_docs)]

#[doc(hidden)]
mod read;
mod ser;
#[cfg(test)]
mod tests;
#[doc(hidden)]
mod write;

use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::Arc;
use std::{fmt, mem, str};

use arcstr::ArcStr;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use chrono::prelude::*;
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use derive_more::{self, Add, AddAssign, Sub, SubAssign};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use serde::{Deserialize, Serialize};
#[macro_use]
extern crate derive_builder;

// Internal Modules
use read::{GdsParser, GdsScanner, GdsStructScan};
pub use ser::{SerdeFile, SerializationFormat};
use write::GdsWriter;

/// An enumeration of GDS record types.
///
/// In the numeric-order specified by GDSII, for automatic [`FromPrimitive`] conversions.
#[derive(FromPrimitive, Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum GdsRecordType {
    Header = 0x00,
    BgnLib,
    LibName,
    Units,
    EndLib,
    BgnStruct,
    StructName, // STRNAME
    EndStruct,
    Boundary,
    Path,
    StructRef,
    ArrayRef,
    Text,
    Layer,
    DataType,
    Width,
    Xy,
    EndElement,
    StructRefName, // SNAME
    ColRow,
    TextNode, // "Not currently used"
    Node,
    TextType,
    Presentation,
    Spacing, // "Discontinued"
    String,
    Strans,
    Mag,
    Angle,
    Uinteger, // "No longer used"
    Ustring,  // "No longer used"
    RefLibs,
    Fonts,
    PathType,
    Generations,
    AttrTable,
    StypTable, // "Unreleased Feature"
    StrType,   // "Unreleased Feature"
    ElemFlags,
    ElemKey,  // "Unreleased Feature"
    LinkType, // "Unreleased Feature"
    LinkKeys, // "Unreleased Feature"
    Nodetype,
    PropAttr,
    PropValue,
    Box,
    BoxType,
    Plex,
    BeginExtn, // "Only occurs in CustomPlus"
    EndExtn,   // "Only occurs in CustomPlus"
    TapeNum,
    TapeCode,
    StrClass, // "Only for Calma internal use"
    Reserved, // "Reserved for future use"
    Format,
    Mask,
    EndMasks,
    LibDirSize,
    SrfName,
    LibSecur,
}

impl GdsRecordType {
    /// Returns whether a record type is valid.
    ///
    /// Many are either deprecated or provisioned without ever being implemented;
    /// all from this list are deemed invalid.
    pub fn valid(&self) -> bool {
        match self {
            Self::TextNode | // "Not currently used"
            Self::Spacing | // "Discontinued"
            Self::Uinteger | // "No longer used"
            Self::Ustring |  // "No longer used"
            Self::StypTable | // "Unreleased Feature"
            Self::StrType |   // "Unreleased Feature"
            Self::ElemKey |   // "Unreleased Feature"
            Self::LinkType |  // "Unreleased Feature"
            Self::LinkKeys |  // "Unreleased Feature"
            Self::StrClass | // "Only for Calma internal use"
            Self::Reserved   // "Reserved for future use"
              => false,
            _ => true,
        }
    }
}

/// An enumeration of GDS data types.
///
/// Listed in order as decoded from 16-bit integers in binary data.
#[derive(FromPrimitive, Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum GdsDataType {
    NoData = 0,
    BitArray = 1,
    I16 = 2,
    I32 = 3,
    F32 = 4,
    F64 = 5,
    Str = 6,
}

/// A GDS record header.
///
/// Decoded contents of a record's four header bytes,
/// including its record-type, data-type, and length in bytes.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
pub struct GdsRecordHeader {
    rtype: GdsRecordType,
    dtype: GdsDataType,
    len: u16,
}

/// An enumeration of GDS record types.
///
/// Keeps each record in relatively "raw" form,
/// other than assuring correct data-types,
/// and converting one-entry arrays into scalars.
/// Invalid record-types are not included.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum GdsRecord {
    Header { version: i16 },
    BgnLib { dates: Vec<i16> }, // Note: always length 12
    LibName(ArcStr),
    Units(f64, f64),
    EndLib,
    BgnStruct { dates: Vec<i16> }, // Note: always length 12
    StructName(ArcStr),            // STRNAME Record
    StructRefName(ArcStr),         // SNAME Record
    EndStruct,
    Boundary,
    Path,
    StructRef,
    ArrayRef,
    Text,
    Layer(i16),
    DataType(i16),
    Width(i32),
    Xy(Vec<i32>),
    EndElement,
    ColRow { cols: i16, rows: i16 },
    Node,
    TextType(i16),
    Presentation(u8, u8),
    String(ArcStr),
    Strans(u8, u8),
    Mag(f64),
    Angle(f64),
    RefLibs(ArcStr),
    Fonts(ArcStr),
    PathType(i16),
    Generations(i16),
    AttrTable(ArcStr),
    ElemFlags(u8, u8),
    Nodetype(i16),
    PropAttr(i16),
    PropValue(ArcStr),
    Box,
    BoxType(i16),
    Plex(i32),
    BeginExtn(i32),
    EndExtn(i32),
    TapeNum(i16),
    TapeCode(Vec<i16>), // Note: always length 6
    Format(i16),
    Mask(ArcStr),
    EndMasks,
    LibDirSize(i16),
    SrfName(ArcStr),
    LibSecur(i16),
}

/// A namespace for encoding and decoding GDSII floats to and from the IEEE754 double-precision
/// format.
pub struct GdsFloat64;

impl GdsFloat64 {
    /// Decodes GDSII's eight-byte representation, stored as a `u64`, to IEEE (and Rust)-compatible `f64`.
    pub fn decode(val: u64) -> f64 {
        // Extract the MSB Sign bit
        let neg = (val & 0x8000_0000_0000_0000) != 0;
        // Extract the 7b exponent
        let exp: i32 = ((val & 0x7F00_0000_0000_0000) >> (8 * 7)) as i32 - 64;
        // Create the initially integer-valued mantissa from the 7 least-significant bytes
        let mantissa: u64 = val & 0x00FF_FFFF_FFFF_FFFF;
        // And apply its normalization to the range (1/16, 1)
        let mantissa: f64 = mantissa as f64 / 2f64.powi(8 * 7);
        // Combine everything into our overall value
        if neg {
            -1.0 * mantissa * 16f64.powi(exp)
        } else {
            mantissa * 16f64.powi(exp)
        }
    }
    /// Encodes `f64` to GDSII's eight bytes, stored as `u64`.
    pub fn encode(mut val: f64) -> u64 {
        if val == 0.0 {
            return 0;
        };
        let mut top: u8 = 0;
        if val < 0.0 {
            top = 0x80;
            val = -val;
        }
        let fexp: f64 = 0.25 * val.log2();
        let mut exponent = fexp.ceil() as i32;
        if fexp == fexp.ceil() {
            exponent += 1;
        }
        let mantissa: u64 = (val * 16_f64.powi(14 - exponent)).round() as u64;
        top += (64 + exponent) as u8;
        let result: u64 = (top as u64).wrapping_shl(56) | (mantissa & 0x00FF_FFFF_FFFF_FFFF);
        result
    }
}

/// A marker type for unsupported features.
#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct Unsupported;

/// A GDS translation setting.
///
/// Reflection, rotation, and magnification for text-elements and references.
/// As configured by `STRANS` records.
#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GdsStrans {
    // Required Fields
    /// Reflection about the x-axis.
    ///
    /// Applied before rotation.
    pub reflected: bool,
    /// Absolute magnification setting
    pub abs_mag: bool,
    /// Absolute angle setting
    pub abs_angle: bool,

    // Optional Fields
    /// Magnification factor. Defaults to 1.0 if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mag: Option<f64>,
    /// Angle in degrees counter-clockwise. Defaults to zero if not specified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub angle: Option<f64>,
}

/// GDS text presentation flags.
///
/// Sets fonts, text justification, and the like.
/// Stored in raw `u8` form.
#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GdsPresentation(u8, u8);

/// GDS element flags.
///
/// As configured by `ELFLAGS` records.
/// Two bytes of bit-fields stored in raw `u8` form.
#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GdsElemFlags(u8, u8);

/// A GDS plex record.
///
/// From the spec:
/// "A unique positive number which is common to all elements of the Plex to which this element belongs."
/// In Gds21's experience, `PLEX` records and settings are highly uncommon.
#[derive(Default, Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GdsPlex(i32);

/// An encoding of GDS units.
///
/// Each GDSII Library has two length-units, referred to as "DB Units" and "User Units" respectively.
/// Essentially all spatial data throughout the Library is denoted in "DB Units".
/// "User units" are a sort of recommendation for GUI programs to use when displaying the Library.  
///
/// From the spec's `UNITS` record-description:  
/// ```text
/// Contains two eight-byte real numbers.
/// The first number is the size of a database-unit, in user-units.
/// The second is the size of a database-unit in meters.
/// To calculate the size of a user-unit in meters, divide the second number by the first.
/// ```
///
/// These two numbers are stored as-is in the [GdsUnits] tuple-struct.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GdsUnits(f64, f64);

impl GdsUnits {
    /// Creates a new [GdsUnits]
    pub const fn new(num1: f64, num2: f64) -> Self {
        Self(num1, num2)
    }
    /// Gets the database-unit size in meters. Used for all spatial data.
    pub const fn db_unit(&self) -> f64 {
        self.1
    }
    /// Gets the user-unit size in meters. Largely for display/debug.
    pub const fn user_unit(&self) -> f64 {
        self.0 / self.1
    }
}

impl Default for GdsUnits {
    /// Default values for GDS units:
    /// * DB-Unit = 1nm
    /// * User-Unit = 1µm (1000x the DB-Unit)
    fn default() -> Self {
        Self(1e-3, 1e-9)
    }
}

/// A GDS spatial point.
///
/// Coordinate in (x,y) layout-space.
/// Denoted in each [GdsLibrary]'s [GdsUnits].
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct GdsPoint {
    pub x: i32,
    pub y: i32,
}

impl GdsPoint {
    /// Creates a new [GdsPoint].
    pub fn new(x: i32, y: i32) -> Self {
        GdsPoint { x, y }
    }

    /// Creates a vector of [GdsPoint] from an array of tuples.
    pub fn vec(pts: &[(i32, i32)]) -> Vec<Self> {
        pts.iter().map(|pt| Self::new(pt.0, pt.1)).collect()
    }

    /// Converts from a two-element slice.
    fn parse(from: &[i32]) -> GdsResult<Self> {
        if from.len() != 2 {
            return Err(GdsError::Str(
                "GdsPoint coordinate vector: Invalid number of elements".into(),
            ));
        }
        Ok(GdsPoint {
            x: from[0],
            y: from[1],
        })
    }

    /// Converts an n-element vector if `i32` into an n/2-element vector of [GdsPoint]s.
    fn parse_vec(from: &[i32]) -> GdsResult<Vec<GdsPoint>> {
        if from.len() % 2 != 0 {
            return Err(GdsError::Str(
                "GdsPoint coordinate vector: Invalid number of elements".into(),
            ));
        }
        let mut rv = Vec::with_capacity(from.len() / 2);
        for i in 0..from.len() / 2 {
            rv.push(GdsPoint {
                x: from[i * 2],
                y: from[i * 2 + 1],
            });
        }
        Ok(rv)
    }

    /// Flattens to a two-element vector
    fn flatten(&self) -> Vec<i32> {
        vec![self.x, self.y]
    }

    /// Converts an n-element slice of [GdsPoint]s to a 2n-element i32 vector.
    fn flatten_vec(src: &[GdsPoint]) -> Vec<i32> {
        let mut rv = Vec::with_capacity(src.len() * 2);
        for pt in src.iter() {
            rv.push(pt.x);
            rv.push(pt.y);
        }
        rv
    }
}

impl std::fmt::Display for GdsPoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// An enumeration of GDS mask formats.
///
/// As set by the FORMAT record.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum GdsFormatType {
    /// Default, sole fully-supported case.
    Archive,
    /// Filtered-format includes a list of Mask records. Not supported.
    Filtered(Vec<Unsupported>),
}

/// A GDS property specification.
///
/// Spec BNF:
/// ```text
/// PROPATTR PROPVALUE
/// ```
#[derive(Default, Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GdsProperty {
    /// Attribute Number
    pub attr: i16,
    /// Attribute Value
    pub value: ArcStr,
}

/// A GDS path element.
///
/// Spec BNF:
/// ```text
/// PATH [ELFLAGS] [PLEX] LAYER DATATYPE [PATHTYPE] [WIDTH] XY [BGNEXTN] [ENDEXTN])
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsPath {
    // Required Fields
    /// Layer number.
    pub layer: i16,
    /// Data type ID.
    pub datatype: i16,
    /// Vector of x,y coordinates.
    pub xy: Vec<GdsPoint>,

    // Optional Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub width: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub path_type: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub begin_extn: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub end_extn: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// A GDS boundary element.
///
/// The most common type for closed-form shapes in GDSII.
/// Most IC layout is comprised of [GdsBoundary] elements, which represent individual polygons.
/// GDSII dictates that the first two and final two coordinates in each [GdsBoundary]
/// shall be identical, "closing" the polygon.
/// Hence an N-sided polygon is represented by an (N+1)-point `xy` vector.
///
/// Spec BNF:
/// ```text
/// BOUNDARY [ELFLAGS] [PLEX] LAYER DATATYPE XY
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsBoundary {
    // Required Fields
    /// Layer number.
    pub layer: i16,
    /// Data type ID.
    pub datatype: i16,
    /// Vector of x,y coordinates.
    pub xy: Vec<GdsPoint>,

    // Optional Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// A GDS struct reference (cell instance).
///
/// Represents an instance of a layout cell.
/// Coordinate vector `xy` is dictated by spec to have exactly one point (or two numbers),
/// specifying the instance's lower-left coordinate.
/// Options for rotation and reflection are configured in the [GdsStrans] attribute `strans`.
///
/// Spec BNF:
/// ```text
/// SREF [ELFLAGS] [PLEX] SNAME [<strans>] XY
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsStructRef {
    // Required Fields
    /// Struct (cell) name.
    pub name: ArcStr,
    /// Location x,y coordinates.
    pub xy: GdsPoint,

    // Optional Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    /// Translation & reflection options.
    pub strans: Option<GdsStrans>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// A GDS array reference.
///
/// A two-dimensional array of struct (cell) instances.
///
/// Spec BNF:
/// ```text
/// AREF [ELFLAGS] [PLEX] SNAME [<strans>] COLROW XY
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsArrayRef {
    // Required Fields
    /// Struct (cell) name.
    pub name: ArcStr,
    /// Vector of x,y coordinates.
    pub xy: [GdsPoint; 3],
    /// Number of columns.
    pub cols: i16,
    /// Number of rows.
    pub rows: i16,
    // Optional Fields
    /// Translation & reflection options.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    pub strans: Option<GdsStrans>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// A GDS text element.
///
/// Spec BNF:
/// ```text
/// TEXT [ELFLAGS] [PLEX] LAYER
/// TEXTTYPE [PRESENTATION] [PATHTYPE] [WIDTH] [<strans>] XY STRING
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsTextElem {
    // Required Fields
    /// Text value.
    pub string: ArcStr,
    /// Layer number.
    pub layer: i16,
    /// Text-type ID.
    pub texttype: i16,
    /// Vector of x,y coordinates.
    pub xy: GdsPoint,

    // Optional Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub presentation: Option<GdsPresentation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub path_type: Option<i16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub width: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default)]
    /// Translation & reflection options.
    pub strans: Option<GdsStrans>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// GDS node element
///
/// Spec BNF:
/// ```text
/// NODE [ELFLAGS] [PLEX] LAYER NODETYPE XY
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsNode {
    // Required Fields
    /// Layer number.
    pub layer: i16,
    /// Node type ID.
    pub nodetype: i16,
    /// Vector of x,y coordinates.
    pub xy: Vec<GdsPoint>,

    // Optional Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// A GDS box element.
///
/// Spec BNF:
/// ```text
/// BOX [ELFLAGS] [PLEX] LAYER BOXTYPE XY
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsBox {
    // Required Fields
    /// Layer number.
    pub layer: i16,
    /// Box type ID.
    pub boxtype: i16,
    /// Vector of x,y coordinates.
    pub xy: [GdsPoint; 5],

    // Optional Fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub elflags: Option<GdsElemFlags>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[builder(default, setter(strip_option))]
    pub plex: Option<GdsPlex>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default, setter(strip_option))]
    pub properties: Vec<GdsProperty>,
}

/// An enumeration of GDS elements.
///
/// Primary union of geometric elements, instances, and arrays which comprise a GDSII struct (cell).
///
/// Spec BNF:
/// ```text
/// {<boundary> | <path> | <SREF> | <AREF> | <text> | <node> | <box>} {<property>}* ENDEL
/// ```
///
/// Note the `properties` vectors are pushed down to each enum variant.
#[derive(derive_more::From, Debug, Clone, Deserialize, Serialize, PartialEq)]
#[allow(missing_docs)]
pub enum GdsElement {
    GdsBoundary(GdsBoundary),
    GdsPath(GdsPath),
    GdsStructRef(GdsStructRef),
    GdsArrayRef(GdsArrayRef),
    GdsTextElem(GdsTextElem),
    GdsNode(GdsNode),
    GdsBox(GdsBox),
}

/// GDS summary stats.
///
/// Summary statistics for a [GdsLibrary] or [GdsStruct].  
/// Total numbers of elements of each type.
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Add, AddAssign, Sub, SubAssign)]
pub struct GdsStats {
    libraries: usize,
    structs: usize,
    boundaries: usize,
    paths: usize,
    struct_refs: usize,
    array_refs: usize,
    text_elems: usize,
    nodes: usize,
    boxes: usize,
}

/// GDS modification dates and times.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct GdsDateTimes {
    /// Last modification date and time.
    pub modified: NaiveDateTime,
    /// Last access date and time.
    pub accessed: NaiveDateTime,
}

impl Default for GdsDateTimes {
    /// Default dates and times.
    fn default() -> Self {
        let now = Utc::now().naive_utc();
        Self {
            modified: now,
            accessed: now,
        }
    }
}

/// A GDS struct (cell) definition
///
/// GDSII's primary hierarchical layout-definition object is its "struct",
/// which most other layout systems would call a "cell" or "module".
/// (Most GDSII software calls them one of these as well.)  
///
/// [GdsStruct]s are principally composed of an un-ordered, un-indexed vector
/// of [GdsElement]s, which can be polygons ([GdsBoundary]),
/// instances of other layouts ([GdsStructRef]),
/// two-dimensional arrays thereof ([GdsArrayRef]),
/// and a handful of other [GdsElement]s.  
///
/// Spec BNF:
/// ```text
/// BGNSTR STRNAME [STRCLASS] {<element>}* ENDSTR
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
pub struct GdsStruct {
    /// Struct name.
    pub name: ArcStr,
    /// Creation/modification date info.
    pub dates: GdsDateTimes,
    /// Elements list.
    pub elems: Vec<GdsElement>,
}
impl GdsStruct {
    /// Creates a new and empty [GdsStruct].
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    /// Counts and returns element statistics.
    fn stats(&self) -> GdsStats {
        let mut stats = GdsStats::default();
        stats.structs += 1;
        for elem in &self.elems {
            use GdsElement::*;
            match elem {
                GdsBoundary(_) => stats.boundaries += 1,
                GdsPath(_) => stats.paths += 1,
                GdsStructRef(_) => stats.struct_refs += 1,
                GdsArrayRef(_) => stats.array_refs += 1,
                GdsTextElem(_) => stats.text_elems += 1,
                GdsNode(_) => stats.nodes += 1,
                GdsBox(_) => stats.boxes += 1,
            };
        }
        stats
    }
}

/// A GDS library.
///
/// The Library is GDSII's primary idiom for a suite of layout-cells.
/// A Library generally corresponds one-to-one with a `.gds` file.
/// Libraries consist primarily of cell-definitions ([GdsStruct]s),
/// and secondarily include library-level meta-data, including the distance units, GDS-spec version, and modification dates.
///
/// Several more esoteric library-level GDSII features are included as [GdsLibrary] fields,
/// but are not materially supported. The empty [Unsupported] value generally denotes these fields.
///
/// Spec BNF:
/// ```text
/// HEADER BGNLIB [LIBDIRSIZE] [SRFNAME] [LIBSECUR] LIBNAME [REFLIBS] [FONTS] [ATTRTABLE] [GENERATIONS] [<FormatType>]
/// UNITS {<structure>}* ENDLIB
/// ```
#[derive(Default, Clone, Builder, Debug, Deserialize, Serialize, PartialEq)]
#[builder(pattern = "owned", setter(into), private)]
#[allow(missing_docs)]
pub struct GdsLibrary {
    // Required fields
    /// Library name.
    pub name: ArcStr,
    /// Gds spec version.
    pub version: i16,
    /// Modification date(s).
    pub dates: GdsDateTimes,
    /// Spatial units.
    pub units: GdsUnits,
    /// Struct definitions.
    pub structs: Vec<GdsStruct>,

    // Unsupported Fields
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub libdirsize: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub srfname: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub libsecur: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub reflibs: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub fonts: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub attrtable: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub generations: Unsupported,
    #[serde(default, skip_serializing)]
    #[builder(default)]
    pub format_type: Unsupported,
}

impl GdsLibrary {
    /// Creates a new and empty [GdsLibrary].
    pub fn new(name: impl Into<ArcStr>) -> Self {
        Self {
            name: name.into(),
            version: 3,
            ..Default::default()
        }
    }

    /// Creates a new and empty [GdsLibrary] with the given units.
    pub fn with_units(name: impl Into<ArcStr>, units: GdsUnits) -> Self {
        Self {
            name: name.into(),
            version: 3,
            units,
            ..Default::default()
        }
    }

    /// Reads a GDS loaded from file at path `fname`.
    pub fn load(fname: impl AsRef<Path>) -> GdsResult<GdsLibrary> {
        // Create the parser, and parse a library.
        GdsParser::open(fname)?.parse_lib()
    }

    /// Reads a [GdsLibrary] from byte-vector `bytes`.
    pub fn from_bytes(bytes: Vec<u8>) -> GdsResult<GdsLibrary> {
        // Create the parser, and parse a Library
        GdsParser::from_bytes(bytes)?.parse_lib()
    }

    /// Runs a first-pass scan of GDSII data in `fname`.
    ///
    /// Returns a vector of [GdsStructScan]s including summary info per struct.
    #[allow(dead_code)] // FIXME!
    fn scan(fname: impl AsRef<Path>) -> GdsResult<Vec<GdsStructScan>> {
        GdsScanner::scan(fname)
    }

    /// Collects and returns the library's aggregate statistics
    /// (numbers of structs, elements by type).
    pub fn stats(&self) -> GdsStats {
        let mut stats = GdsStats::default();
        stats.libraries += 1;
        for strukt in self.structs.iter() {
            stats += strukt.stats();
        }
        stats
    }

    /// Saves to file at path `fname`.
    pub fn save(&self, fname: impl AsRef<Path>) -> GdsResult<()> {
        if let Some(prefix) = fname.as_ref().parent() {
            std::fs::create_dir_all(prefix)?;
        }
        let mut wr = GdsWriter::open(fname)?;
        wr.write_lib(self)
    }

    /// Writes to file `file`.
    pub fn write(&self, file: impl Write) -> GdsResult<()> {
        let mut wr = GdsWriter::new(file);
        wr.write_lib(self)
    }
}

// Enable [GdsLibrary] and [GdsStruct] serialization to file, in each of `utils` supported formats.
impl SerdeFile for GdsLibrary {}
impl SerdeFile for GdsStruct {}

/// A GDS layer spec.
///
/// Each GDSII element's layer is specified by a set of two numbers,
/// commonly referred to as `layer` and `datatype`.
/// Several element-types refer to their analog of `datatype` by different names,
/// e.g. `texttype` and `nodetype`.  
///
/// `GdsLayerSpecs` generalize across these via the `xtype` field,
/// which holds whichever is appropriate for the given element.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GdsLayerSpec {
    /// Layer ID number.
    pub layer: i16,
    /// Data type (or text type, node type, etc.) ID number.
    pub xtype: i16,
}

/// An object that has a corresponding GDS layer.
pub trait HasLayer {
    /// Returns a [GdsLayerSpec] including the two numbers `layer` and `xtype`.
    fn layerspec(&self) -> GdsLayerSpec;
}

impl GdsLayerSpec {
    /// Creates a new [GdsLayerSpec].
    pub fn new(layer: i16, xtype: i16) -> GdsLayerSpec {
        GdsLayerSpec { layer, xtype }
    }
}

impl HasLayer for GdsBoundary {
    fn layerspec(&self) -> GdsLayerSpec {
        GdsLayerSpec::new(self.layer, self.datatype)
    }
}

impl HasLayer for GdsTextElem {
    fn layerspec(&self) -> GdsLayerSpec {
        GdsLayerSpec::new(self.layer, self.texttype)
    }
}

impl HasLayer for GdsNode {
    fn layerspec(&self) -> GdsLayerSpec {
        GdsLayerSpec::new(self.layer, self.nodetype)
    }
}

impl HasLayer for GdsBox {
    fn layerspec(&self) -> GdsLayerSpec {
        GdsLayerSpec::new(self.layer, self.boxtype)
    }
}

impl HasLayer for GdsPath {
    fn layerspec(&self) -> GdsLayerSpec {
        GdsLayerSpec::new(self.layer, self.datatype)
    }
}

/// An enumeration of each context in which a record can be parsed, primarily for error reporting.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum GdsContext {
    Library,
    Struct,
    StructRef,
    ArrayRef,
    Boundary,
    Box,
    Path,
    Text,
    Node,
    Property,
}

/// A result type alias.
pub type GdsResult<T> = Result<T, GdsError>;

/// An enumeration of GDS errors.
///
/// Most errors are tied in some sense to parsing and decoding.
/// Once a valid [GdsLibrary] is created in memory, it can generally be streamed to bytes.
#[derive(Debug, Clone)]
pub enum GdsError {
    /// Invalid binary -> record conversion.
    RecordDecode(GdsRecordType, GdsDataType, u16),
    /// Invalid record length.
    RecordLen(usize),
    /// Invalid data type.
    InvalidDataType(u8),
    /// Invalid record type.
    InvalidRecordType(u8),
    /// Unsupported feature, in the decoded context.
    Unsupported(Option<GdsRecord>, Option<GdsContext>),
    /// Parser errors.
    #[allow(missing_docs)]
    Parse {
        msg: String,
        record: GdsRecord,
        recordnum: usize,
        bytepos: u64,
        ctx: Vec<GdsContext>,
    },
    /// Boxed (external) errors.
    Boxed(Arc<dyn Error + Send + Sync>),
    /// Other errors.
    Str(String),
}

impl std::fmt::Display for GdsError {
    /// Displays a [GdsError].
    ///
    /// This functionally delegates to the (derived) [std::fmt::Debug] implementation.
    /// Maybe more info that wanted in some cases. But certainly enough.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for GdsError {}

impl From<std::io::Error> for GdsError {
    fn from(e: std::io::Error) -> Self {
        Self::Boxed(Arc::new(e))
    }
}

impl From<std::str::Utf8Error> for GdsError {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Boxed(Arc::new(e))
    }
}

impl From<String> for GdsError {
    fn from(e: String) -> Self {
        GdsError::Str(e)
    }
}

impl From<&str> for GdsError {
    fn from(e: &str) -> Self {
        GdsError::Str(e.to_string())
    }
}

#[cfg(any(test, feature = "selftest"))]
/// Check `lib` matches across a write-read round-trip cycle
pub fn roundtrip(lib: &GdsLibrary) -> GdsResult<()> {
    use tempfile::tempfile;

    // Write to a temporary file
    let mut file = tempfile()?;
    lib.write(&mut file)?;

    // Rewind to the file-start, and read it back
    file.rewind()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let lib2 = GdsLibrary::from_bytes(bytes)?;

    // And check the two line up
    assert_eq!(*lib, lib2);
    Ok(())
}
