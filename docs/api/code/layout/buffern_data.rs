#[derive(Default, LayoutData)]
pub struct BufferNData {
    #[substrate(transform)]
    pub buffers: Vec<Instance<Buffer>>,
    pub width: i64,
}
