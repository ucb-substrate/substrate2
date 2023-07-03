impl HasLayout for Inverter {
    type Data = ();
}

impl HasLayoutImpl<ExamplePdk> for Inverter {
    fn layout(
        &self,
        io: &mut <<Self as substrate::block::Block>::Io as substrate::io::LayoutType>::Builder,
        cell: &mut substrate::layout::CellBuilder<ExamplePdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(cell.ctx.pdk.layers.met1, Rect::from_sides(0, 0, 100, 200)));
        Ok(())
    }
}

