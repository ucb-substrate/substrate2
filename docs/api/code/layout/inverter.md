impl HasLayout for Inverter {
    type Data = ();
}

impl HasLayoutImpl<ExamplePdk> for Inverter {
    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<ExamplePdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(Rect::from_sides(0, 0, 100, 200)));
        Ok(())
    }
}

