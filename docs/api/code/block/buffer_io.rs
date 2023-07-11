#[derive(Io, Clone, Default)]
pub struct BufferIo {
    vdd: InOut<Signal>,
    vss: InOut<Signal>,
    #[substrate(layout_type = "ShapePort")]
    din: Input<Signal>,
    #[substrate(layout_type = "ShapePort")]
    dout: Output<Signal>,
}
