//! An enumeration of supported corners.

use std::path::PathBuf;

use crate::{Sky130Pdk, SpectreModelSelect};
use arcstr::ArcStr;
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

impl Sky130Pdk {
    fn spectre_model_file_includes(&self, corner: Sky130Corner) -> Vec<(PathBuf, Vec<ArcStr>)> {
        match self.spectre_model_select {
            SpectreModelSelect::All => [
                SpectreModelSelect::SrcNda,
                SpectreModelSelect::Cds,
                SpectreModelSelect::Open,
            ]
            .into_iter()
            .filter_map(|select| self.spectre_model_file_includes_helper(select, corner))
            .collect(),
            select => self
                .spectre_model_file_includes_helper(select, corner)
                .into_iter()
                .collect(),
        }
    }
    fn spectre_model_file_includes_helper(
        &self,
        select: SpectreModelSelect,
        corner: Sky130Corner,
    ) -> Option<(PathBuf, Vec<ArcStr>)> {
        Some(match select {
            SpectreModelSelect::All => unreachable!(),
            SpectreModelSelect::SrcNda => (
                self.src_nda_root_dir
                    .as_ref()?
                    .join("MODELS/SPECTRE/s8phirs_10r/Models/design_wrapper.lib.scs"),
                vec![
                    arcstr::format!("{}_fet", corner.name()),
                    arcstr::format!("{}_cell", corner.name()),
                    arcstr::format!("{}_parRC", corner.name()),
                    arcstr::format!("{}_rc", corner.name()),
                ],
            ),
            SpectreModelSelect::Cds => (
                self.cds_root_dir.as_ref()?.join("models/sky130.lib.spice"),
                vec![corner.name()],
            ),
            SpectreModelSelect::Open => (
                self.open_root_dir
                    .as_ref()?
                    .join("libraries/sky130_fd_pr/latest/models/sky130.lib.spice"),
                vec![corner.name()],
            ),
        })
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
        for (path, sections) in pdk.spectre_model_file_includes(self) {
            for section in sections {
                opts.include_section(&path, section);
            }
        }
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
                .expect("Open root directory must be specified")
                .join("libraries/sky130_fd_pr/latest/models/sky130.lib.spice"),
            self.name(),
        )
    }
}
