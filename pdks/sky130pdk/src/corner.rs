//! An enumeration of supported corners.

use crate::Sky130Pdk;
use ngspice::Ngspice;
use serde::{Deserialize, Serialize};
use spectre::Spectre;
use substrate::simulation::options::SimOption;
use substrate::simulation::{SimulationContext, Simulator};

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

impl Sky130Corner {
    /// Returns the name of the corner.
    pub fn name(&self) -> arcstr::ArcStr {
        match *self {
            Self::Tt => arcstr::literal!("tt"),
            Self::Fs => arcstr::literal!("fs"),
            Self::Sf => arcstr::literal!("sf"),
            Self::Ff => arcstr::literal!("ff"),
            Self::Ss => arcstr::literal!("ss"),
        }
    }
}

impl SimOption<Spectre> for Sky130Corner {
    fn set_option(
        self,
        opts: &mut <Spectre as Simulator>::Options,
        ctx: &SimulationContext<Spectre>,
    ) {
        let pdk = ctx
            .ctx
            .get_installation::<Sky130Pdk>()
            .expect("Sky130 PDK must be installed");
        let design_wrapper_path = pdk
            .commercial_root_dir
            .as_ref()
            .expect("Commercial root directory must be specified")
            .join("MODELS/SPECTRE/s8phirs_10r/Models/design_wrapper.lib.scs");
        opts.include_section(&design_wrapper_path, format!("{}_fet", self.name()));
        opts.include_section(&design_wrapper_path, format!("{}_cell", self.name()));
        opts.include_section(&design_wrapper_path, format!("{}_parRC", self.name()));
        opts.include_section(&design_wrapper_path, format!("{}_rc", self.name()));
    }
}

impl SimOption<Ngspice> for Sky130Corner {
    fn set_option(
        self,
        opts: &mut <Ngspice as Simulator>::Options,
        ctx: &SimulationContext<Ngspice>,
    ) {
        let pdk = ctx
            .ctx
            .get_installation::<Sky130Pdk>()
            .expect("Sky130 PDK must be installed");
        opts.include_section(
            pdk.open_root_dir
                .as_ref()
                .expect("Commercial root directory must be specified")
                .join("libraries/sky130_fd_pr/latest/models/sky130.lib.spice"),
            self.name(),
        )
    }
}
