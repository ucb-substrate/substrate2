#[derive(Default, LayoutData)]
pub struct BufferNData {
    #[transform]
    pub buffers: Vec<Instance<Buffer>>,
    pub width: i64,
}
