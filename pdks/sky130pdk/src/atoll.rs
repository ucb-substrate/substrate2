use crate::layers::Sky130Layers;
use atoll::grid::{AbstractLayer, LayerStack, PdkLayer};
use atoll::RoutingDir;
use substrate::geometry::dir::Dir;
use substrate::pdk::layers::Layer;

impl Sky130Layers {
    /// Returns the ATOLL-compatible routing layer stack.
    pub fn atoll_layer_stack(&self) -> LayerStack<PdkLayer> {
        LayerStack {
            layers: vec![
                PdkLayer {
                    id: self.li1.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Any {
                            track_dir: Dir::Vert,
                        },
                        line: 200,
                        space: 200,
                        offset: 100,
                        endcap: 0,
                    },
                },
                PdkLayer {
                    id: self.met1.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 260,
                        space: 140,
                        offset: 130,
                        endcap: 100,
                    },
                },
                PdkLayer {
                    id: self.met2.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 300,
                        space: 300,
                        offset: 150,
                        endcap: 130,
                    },
                },
                PdkLayer {
                    id: self.met3.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 400,
                        space: 400,
                        offset: 200,
                        endcap: 150,
                    },
                },
                PdkLayer {
                    id: self.met4.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Vert,
                        line: 1_200,
                        space: 1_200,
                        offset: 600,
                        endcap: 200,
                    },
                },
                PdkLayer {
                    id: self.met5.drawing.id(),
                    inner: AbstractLayer {
                        dir: RoutingDir::Horiz,
                        line: 1_800,
                        space: 1_800,
                        offset: 900,
                        endcap: 600,
                    },
                },
            ],
            offset_x: 0,
            offset_y: 0,
        }
    }
}
