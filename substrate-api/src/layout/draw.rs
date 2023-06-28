//! Traits for drawing layout elements.

use crate::error::Result;

use super::element::{Element, Shape};

/// An object that layout elements can be drawn in.
pub trait DrawContainer {
    /// Draws a basic layout element.
    fn draw_element(&mut self, element: Element);

    /// Draws a blockage.
    fn draw_blockage(&mut self, shape: Shape);

    /// Draws an arbitrary drawable object.
    fn draw(&mut self, obj: impl Draw) {
        obj.draw(self);
    }
    /// Draws an arbitrary drawable object from a reference.
    fn draw_ref(&mut self, obj: &impl DrawRef) {
        obj.draw_ref(self);
    }
}

/// An object that can be drawn in a [`DrawContainer`].
pub trait Draw {
    /// Draws `self` inside `container`.
    fn draw<T: DrawContainer + ?Sized>(self, container: &mut T) -> Result<()>;
}

/// An object that can be drawn in a [`DrawContainer`] from its reference.
pub trait DrawRef {
    /// Draws `self` inside `container` from its reference.
    fn draw_ref<T: DrawContainer + ?Sized>(&self, container: &mut T) -> Result<()>;
}

impl<E: Into<Element>> Draw for E {
    fn draw<T: DrawContainer + ?Sized>(self, container: &mut T) -> Result<()> {
        container.draw_element(self.into());
        Ok(())
    }
}
