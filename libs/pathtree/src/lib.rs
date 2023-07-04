//! An immutable tree data structure for fast path operations.
//!
//! [`PathTree`] is inexpensive to clone and supports prepending and appending paths to one
//! another. Useful when several objects in a tree need to store their path relative to the root.
#![warn(missing_docs)]

use std::sync::Arc;

#[cfg(test)]
mod tests;

/// A tree data structure for storing paths without mutation.
///
/// Cloning the data structure is inexpensive since each clone only clones a single [`Arc`].
///
/// # Examples
///
/// ```
/// use pathtree::PathTree;
///
///
/// let path = PathTree::empty();
/// let path = path.append_segment(7);
/// let path = path.append_segment(5);
/// let path = path.prepend_segment(6);
/// let path = path.prepend_segment(8);
///
/// let path_vec: Vec<_> = path.iter().copied().collect();
/// assert_eq!(path_vec, vec![8, 6, 7, 5]);
///
/// let other_path = PathTree::empty();
///
/// let other_path = other_path.append_segment(2);
/// let other_path = other_path.prepend_segment(1);
/// let other_path = other_path.append_segment(3);
/// let other_path = other_path.prepend_segment(4);
///
/// let full_path = other_path.append(&path);
/// let full_path_vec: Vec<_> = full_path.iter().copied().collect();
/// assert_eq!(full_path_vec, vec![4, 1, 2, 3, 8, 6, 7, 5]);
/// ```
#[derive(Debug, Default, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PathTree<T>(Arc<PathTreeInner<T>>);

impl<T> Clone for PathTree<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, Default, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
struct PathTreeInner<T> {
    segment: Option<T>,
    parent: Option<PathTree<T>>,
    child: Option<PathTree<T>>,
}

impl<T> PathTree<T> {
    fn node(segment: Option<T>, parent: Option<PathTree<T>>, child: Option<PathTree<T>>) -> Self {
        PathTree(Arc::new(PathTreeInner {
            segment,
            parent,
            child,
        }))
    }

    fn segment(&self) -> &Option<T> {
        &self.0.segment
    }

    fn parent(&self) -> &Option<Self> {
        &self.0.parent
    }

    fn child(&self) -> &Option<Self> {
        &self.0.child
    }

    /// Creates a new [`PathTree`] with a single segment.
    pub fn new(segment: T) -> Self {
        Self::node(Some(segment), None, None)
    }

    /// Creates an empty [`PathTree`].
    pub fn empty() -> Self {
        Self::node(None, None, None)
    }

    /// Appends another [`PathTree`] to `self`.
    pub fn append(&self, other: &PathTree<T>) -> Self {
        Self::node(None, Some((*self).clone()), Some((*other).clone()))
    }

    /// Prepends another [`PathTree`] to `self`.
    pub fn prepend(&self, other: &PathTree<T>) -> Self {
        other.append(self)
    }

    /// Appends a segment to `self`.
    pub fn append_segment(&self, segment: T) -> Self {
        Self::node(Some(segment), Some((*self).clone()), None)
    }

    /// Prepends a segment to `self`.
    pub fn prepend_segment(&self, segment: T) -> Self {
        Self::node(Some(segment), None, Some((*self).clone()))
    }

    /// Creates an iterator over the items in the path.
    pub fn iter(&self) -> Iter<'_, T> {
        let mut iter = Iter::new();

        iter.push_node_and_parents(self);

        iter
    }
}

impl<'a, T> IntoIterator for &'a PathTree<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A> FromIterator<A> for PathTree<A> {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut iter = iter.into_iter();
        if let Some(elem) = iter.next() {
            let mut tree = PathTree::new(elem);
            for elem in iter {
                tree = tree.append_segment(elem);
            }
            tree
        } else {
            PathTree::empty()
        }
    }
}

/// An iterator over a [`PathTree`].
pub struct Iter<'a, T> {
    stack: Vec<&'a PathTree<T>>,
}

impl<'a, T> Iter<'a, T> {
    fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn push_node_and_parents(&mut self, node: &'a PathTree<T>) {
        let mut parent = Some(node);
        while let Some(curr) = parent {
            self.stack.push(curr);
            parent = curr.parent().as_ref();
        }
    }

    fn push_child(&mut self, node: &'a PathTree<T>) {
        if let Some(child) = node.child() {
            self.push_node_and_parents(child);
        }
    }

    fn pop(&mut self) -> Option<&'a PathTree<T>> {
        let top = self.stack.pop();
        if let Some(top) = top {
            self.push_child(top);
        }
        top
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.pop() {
            if let Some(segment) = next.segment().as_ref() {
                return Some(segment);
            }
        }

        None
    }
}
