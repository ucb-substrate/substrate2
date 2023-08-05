# Pathtree

An immutable tree data structure for fast path operations.

`PathTree` is inexpensive to clone and supports prepending and appending paths to one
another. Useful when several objects in a tree need to store their path relative to the root.

# Usage

```rust
use pathtree::PathTree;


let path = PathTree::empty();
let path = path.append_segment(7);
let path = path.append_segment(5);
let path = path.prepend_segment(6);
let path = path.prepend_segment(8);

let path_vec: Vec<_> = path.iter().copied().collect();
assert_eq!(path_vec, vec![8, 6, 7, 5]);

let other_path = PathTree::empty();

let other_path = other_path.append_segment(2);
let other_path = other_path.prepend_segment(1);
let other_path = other_path.append_segment(3);
let other_path = other_path.prepend_segment(4);

let full_path = other_path.append(&path);
let full_path_vec: Vec<_> = full_path.iter().copied().collect();
assert_eq!(full_path_vec, vec![4, 1, 2, 3, 8, 6, 7, 5]);
```
