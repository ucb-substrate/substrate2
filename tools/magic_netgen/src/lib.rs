use magic::extract::{ExtractParams, run_extract};
use magic::pex::{PexParams, run_pex};
use netgen::compare::{CompareOutput, CompareParams};
use scir::NetlistLibConversion;
use scir::netlist::ConvertibleNetlister;
use scir::{NamedSliceOne, SliceOnePath};
use spice::Spice;
use spice::netlist::NetlistOptions;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use substrate::context::Context;
use substrate::schematic::conv::{ConvertedNodePath, RawLib};
use substrate::schematic::{
    Cell, ContextView, HasContextView, HasNestedView, InstancePath, NestedView, PrimitiveBinding,
    Schematic,
};
use substrate::simulation::data::{Save, SaveKey, Saved};
use substrate::simulation::{Analysis, Simulator};
use substrate::types::schematic::{NestedNode, RawNestedNode};
use substrate::types::{Flatten, HasBundleKind, HasNameTree};
use substrate::{
    arcstr::{self, ArcStr},
    block::Block,
};

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct Pex<T> {
    pub schematic: Arc<T>,
    pub gds_path: PathBuf,
    pub layout_cell_name: ArcStr,
    pub work_dir: PathBuf,
    pub magic_tech_file_path: PathBuf,
    pub netgen_setup_file_path: PathBuf,
}

impl<T> Clone for Pex<T> {
    fn clone(&self) -> Self {
        Self {
            schematic: self.schematic.clone(),
            gds_path: self.gds_path.clone(),
            layout_cell_name: self.layout_cell_name.clone(),
            work_dir: self.work_dir.clone(),
            magic_tech_file_path: self.magic_tech_file_path.clone(),
            netgen_setup_file_path: self.netgen_setup_file_path.clone(),
        }
    }
}

impl<T: Block> Block for Pex<T> {
    type Io = <T as Block>::Io;

    fn name(&self) -> ArcStr {
        self.schematic.name()
    }

    fn io(&self) -> Self::Io {
        self.schematic.io()
    }
}

impl<T: Schematic<Schema = Spice>> Schematic for Pex<T>
where
    T::NestedData: HasContextView<PexContext>,
{
    type Schema = Spice;
    type NestedData = PexData<T>;

    fn schematic(
        &self,
        io: &substrate::types::schematic::IoNodeBundle<Self>,
        cell: &mut substrate::schematic::CellBuilder<<Self as Schematic>::Schema>,
    ) -> substrate::error::Result<Self::NestedData> {
        let source_path = self.work_dir.join("source.spice");
        let pex_netlist_path = self
            .work_dir
            .join(format!("{}.pex.spice", self.schematic.name()));
        let lvs_netlist_path = self
            .work_dir
            .join(format!("{}.lvs.spice", self.schematic.name()));
        let rawlib = cell.ctx().export_scir(self.schematic.clone()).unwrap();

        let mut conv = Spice.write_scir_netlist_to_file(
            &rawlib.scir,
            &source_path,
            NetlistOptions::default(),
        )?;
        for (_, cell) in conv.cells.iter_mut() {
            for (_, inst_name) in cell.instances.iter_mut() {
                let mut iter = inst_name.chars();
                iter.next();
                *inst_name = iter.as_str().into();
            }
        }

        let extract_dir = self.work_dir.join("extract");
        let compare_dir = self.work_dir.join("compare");
        let pex_dir = self.work_dir.join("pex");

        run_extract(&ExtractParams {
            cell_name: &self.layout_cell_name,
            work_dir: &extract_dir,
            gds_path: &self.gds_path,
            tech_file_path: &self.magic_tech_file_path,
            netlist_path: &lvs_netlist_path,
        })
        .expect("failed to extract layout for LVS");

        let params = CompareParams {
            netlist1_path: &source_path,
            cell1: &self.schematic.name(),
            netlist2_path: &lvs_netlist_path,
            cell2: &self.layout_cell_name,
            work_dir: &compare_dir,
            setup_file_path: &self.netgen_setup_file_path,
        };
        let output = netgen::compare::compare(&params)
            .expect("failed to compare schematic and layout netlists");

        assert!(output.matches, "LVS failed");

        run_pex(&PexParams {
            cell_name: &self.layout_cell_name,
            work_dir: &pex_dir,
            gds_path: &self.gds_path,
            tech_file_path: &self.magic_tech_file_path,
            pex_netlist_path: &pex_netlist_path,
        })
        .expect("failed to run PEX");

        let cell_inner = cell
            .ctx()
            .generate_schematic(self.schematic.clone())
            .cell()
            .clone();

        // Read the PEX netlist to determine order of ports in the PEX netlist
        // (which may be different than the order of ports in the schematic netlist).
        let spice =
            spice::parser::Parser::parse_file(spice::parser::Dialect::Spice, &pex_netlist_path)
                .expect("failed to parse pex netlist");
        let subckt = spice
            .ast
            .elems
            .iter()
            .filter_map(|e| {
                if let spice::parser::Elem::Subckt(s) = e {
                    Some(s)
                } else {
                    None
                }
            })
            .find(|s| s.name.as_str() == self.layout_cell_name.as_str())
            .expect("did not find layout cell in pex netlist");

        let primitive = spice::Primitive::RawInstanceWithInclude {
            // Magic PEX uses the layout cell name as the name of the extracted subcircuit,
            // not the name of the schematic cell.
            cell: self.layout_cell_name.clone(),
            netlist: pex_netlist_path,
            ports: subckt
                .ports
                .iter()
                .map(|p| ArcStr::from(p.as_str()))
                .collect(),
        };

        let ports = self
            .io()
            .kind()
            .flat_names(None)
            .into_iter()
            .map(|n| arcstr::format!("{}", n))
            .collect::<Vec<ArcStr>>();

        let mut binding = PrimitiveBinding::new(primitive);
        for (n, name) in io.flatten_vec().iter().zip(ports.iter()) {
            binding.connect(name, n);
        }
        cell.set_primitive(binding);
        Ok(PexData {
            cell: cell_inner,
            lib: Arc::new(rawlib),
            conv: Arc::new(conv),
            node_map: Arc::new(output.node_map),
        })
    }
}

pub struct PexContext {
    /// The source spice file for this DSPF extracted view.
    lib: Arc<RawLib<Spice>>,
    conv: Arc<NetlistLibConversion>,
    path: InstancePath,
    node_map: Arc<HashMap<ArcStr, ArcStr>>,
}

impl HasContextView<PexContext> for NestedNode {
    type ContextView = RawNestedNode;

    fn context_view(&self, parent: &PexContext) -> ContextView<Self, PexContext> {
        let n = self;
        let path = parent.lib.convert_node_path(&n.path()).unwrap();
        let path = match path {
            ConvertedNodePath::Cell(path) => path,
            ConvertedNodePath::Primitive {
                instances, port, ..
            } => SliceOnePath::new(instances.clone(), NamedSliceOne::new(port.clone())),
        };
        let path = parent.lib.scir.simplify_path(path);
        let schematic_path =
            Spice::node_path_with_separator(&parent.lib.scir, &parent.conv, &path, "/");
        let layout_path = parent.node_map[schematic_path.as_str()].clone();
        RawNestedNode::new(parent.path.clone(), layout_path)
    }
}

pub struct PexData<T: Schematic> {
    cell: Cell<Arc<T>>,
    lib: Arc<RawLib<Spice>>,
    conv: Arc<NetlistLibConversion>,
    node_map: Arc<HashMap<ArcStr, ArcStr>>,
}

pub struct NestedPexData<T: Schematic> {
    cell: Cell<Arc<T>>,
    ctx: PexContext,
}

impl<T: Schematic> NestedPexData<T>
where
    T::NestedData: HasContextView<PexContext>,
{
    pub fn data(&self) -> ContextView<T::NestedData, PexContext> {
        self.cell.context_data(&self.ctx)
    }
}

impl<T: Schematic> HasNestedView for PexData<T> {
    type NestedView = NestedPexData<T>;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedPexData {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.lib.clone(),
                conv: self.conv.clone(),
                path: parent.clone(),
                node_map: self.node_map.clone(),
            },
        }
    }
}

impl<T: Schematic> HasNestedView for NestedPexData<T> {
    type NestedView = NestedPexData<T>;
    fn nested_view(&self, parent: &InstancePath) -> NestedView<Self> {
        NestedPexData {
            cell: self.cell.clone(),
            ctx: PexContext {
                lib: self.ctx.lib.clone(),
                conv: self.ctx.conv.clone(),
                path: self.ctx.path.prepend(parent),
                node_map: self.ctx.node_map.clone(),
            },
        }
    }
}

impl<S: Simulator, A: Analysis, T: Schematic> Save<S, A> for NestedPexData<T>
where
    T::NestedData: HasContextView<PexContext>,
    ContextView<T::NestedData, PexContext>: Save<S, A>,
{
    type SaveKey = SaveKey<ContextView<T::NestedData, PexContext>, S, A>;
    type Saved = Saved<ContextView<T::NestedData, PexContext>, S, A>;

    fn save(
        &self,
        ctx: &substrate::simulation::SimulationContext<S>,
        opts: &mut <S as Simulator>::Options,
    ) -> <Self as Save<S, A>>::SaveKey {
        self.data().save(ctx, opts)
    }

    fn from_saved(
        output: &<A as Analysis>::Output,
        key: &<Self as Save<S, A>>::SaveKey,
    ) -> <Self as Save<S, A>>::Saved {
        <ContextView<T::NestedData, PexContext> as Save<S, A>>::from_saved(output, key)
    }
}

#[derive(Clone)]
pub struct LvsParams<T> {
    pub schematic: Arc<T>,
    pub gds_path: PathBuf,
    pub layout_cell_name: ArcStr,
    pub work_dir: PathBuf,
    pub magic_tech_file_path: PathBuf,
    pub netgen_setup_file_path: PathBuf,
    pub ctx: Context,
}

pub fn run_lvs<T>(params: LvsParams<T>) -> substrate::error::Result<CompareOutput>
where
    T: Schematic<Schema = Spice>,
{
    let source_path = params.work_dir.join("source.spice");
    let lvs_netlist_path = params
        .work_dir
        .join(format!("{}.lvs.spice", params.schematic.name()));
    let rawlib = params.ctx.export_scir(params.schematic.clone()).unwrap();

    let mut conv =
        Spice.write_scir_netlist_to_file(&rawlib.scir, &source_path, NetlistOptions::default())?;
    for (_, cell) in conv.cells.iter_mut() {
        for (_, inst_name) in cell.instances.iter_mut() {
            let mut iter = inst_name.chars();
            iter.next();
            *inst_name = iter.as_str().into();
        }
    }

    let extract_dir = params.work_dir.join("extract");
    let compare_dir = params.work_dir.join("compare");

    run_extract(&ExtractParams {
        cell_name: &params.layout_cell_name,
        work_dir: &extract_dir,
        gds_path: &params.gds_path,
        tech_file_path: &params.magic_tech_file_path,
        netlist_path: &lvs_netlist_path,
    })
    .expect("failed to extract layout for LVS");

    let params = CompareParams {
        netlist1_path: &source_path,
        cell1: &params.schematic.name(),
        netlist2_path: &lvs_netlist_path,
        cell2: &params.layout_cell_name,
        work_dir: &compare_dir,
        setup_file_path: &params.netgen_setup_file_path,
    };
    let output =
        netgen::compare::compare(&params).expect("failed to compare schematic and layout netlists");

    Ok(output)
}
