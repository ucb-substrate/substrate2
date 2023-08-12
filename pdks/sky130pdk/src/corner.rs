//! An enumeration of supported corners.
use ngspice::Ngspice;
use serde::{Deserialize, Serialize};
use spectre::Spectre;
use substrate::pdk::corner::InstallCorner;

use crate::{Sky130CommercialPdk, Sky130OpenPdk};

/// An enumeration of supported corners.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum Sky130Corner {
    /// Typical.
    #[default]
    Tt,
    /// Slow-fast.
    Sf,
    /// Fast-slow.
    Fs,
    /// Fast-fast.
    Ff,
    /// Slow-slow.
    Ss,
}

/// A struct containing each of the [`Sky130Corner`] variants.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Sky130Corners {
    pub(super) tt: Sky130Corner,
    pub(super) sf: Sky130Corner,
    pub(super) fs: Sky130Corner,
    pub(super) ff: Sky130Corner,
    pub(super) ss: Sky130Corner,
}

impl Default for Sky130Corners {
    fn default() -> Self {
        Self::new()
    }
}

impl Sky130Corners {
    /// Creates a new [`Sky130Corners`].
    pub fn new() -> Self {
        Self {
            tt: Sky130Corner::Tt,
            sf: Sky130Corner::Sf,
            fs: Sky130Corner::Fs,
            ff: Sky130Corner::Ff,
            ss: Sky130Corner::Ss,
        }
    }

    /// Returns the typical corner.
    pub fn tt(&self) -> Sky130Corner {
        self.tt
    }
    /// Returns the slow-fast corner.
    pub fn sf(&self) -> Sky130Corner {
        self.sf
    }
    /// Returns the fast-slow corner.
    pub fn fs(&self) -> Sky130Corner {
        self.fs
    }
    /// Returns the fast-fast corner.
    pub fn ff(&self) -> Sky130Corner {
        self.ff
    }
    /// Returns the slow-slow corner.
    pub fn ss(&self) -> Sky130Corner {
        self.ss
    }
}

impl Sky130Corner {
    fn name(&self) -> arcstr::ArcStr {
        match *self {
            Self::Tt => arcstr::literal!("tt"),
            Self::Fs => arcstr::literal!("fs"),
            Self::Sf => arcstr::literal!("sf"),
            Self::Ff => arcstr::literal!("ff"),
            Self::Ss => arcstr::literal!("ss"),
        }
    }
}

impl InstallCorner<Spectre> for Sky130CommercialPdk {
    fn install_corner(
        &self,
        corner: &<Self as substrate::pdk::Pdk>::Corner,
        opts: &mut <Spectre as substrate::simulation::Simulator>::Options,
    ) {
        opts.include(self.root_dir.join(format!(
            "MODELS/SPECTRE/s8phirs_10r/Models/{}.cor",
            corner.name()
        )));
    }
}

impl InstallCorner<Ngspice> for Sky130OpenPdk {
    fn install_corner(
        &self,
        corner: &<Self as substrate::pdk::Pdk>::Corner,
        opts: &mut <Ngspice as substrate::simulation::Simulator>::Options,
    ) {
        opts.include_section(
            self.root_dir
                .join("libraries/sky130_fd_pr/latest/models/sky130.lib.spice"),
            corner.name().clone(),
        );
    }
}
