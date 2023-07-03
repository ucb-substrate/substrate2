use crate::PathTree;

#[test]
fn append_works() {
    let path = PathTree::empty();

    let path = path.append_segment(5);
    let path = path.append_segment(6);
    let path = path.append_segment(7);
    let path = path.append_segment(8);

    let path_vec: Vec<_> = path.iter().copied().collect();
    assert_eq!(path_vec, (5..9).collect::<Vec<_>>());

    let other_path = PathTree::empty();

    let other_path = other_path.append_segment(1);
    let other_path = other_path.append_segment(2);
    let other_path = other_path.append_segment(3);
    let other_path = other_path.append_segment(4);

    let full_path = other_path.append(&path);
    let full_path_vec: Vec<_> = full_path.iter().copied().collect();
    assert_eq!(full_path_vec, (1..9).collect::<Vec<_>>());
}

#[test]
fn prepend_works() {
    let path = PathTree::empty();

    let path = path.prepend_segment(5);
    let path = path.prepend_segment(6);
    let path = path.prepend_segment(7);
    let path = path.prepend_segment(8);

    let path_vec: Vec<_> = path.iter().copied().collect();
    assert_eq!(path_vec, (5..9).rev().collect::<Vec<_>>());

    let other_path = PathTree::empty();

    let other_path = other_path.prepend_segment(1);
    let other_path = other_path.prepend_segment(2);
    let other_path = other_path.prepend_segment(3);
    let other_path = other_path.prepend_segment(4);

    let full_path = other_path.prepend(&path);
    let full_path_vec: Vec<_> = full_path.iter().copied().collect();
    assert_eq!(full_path_vec, (1..9).rev().collect::<Vec<_>>());
}

#[test]
fn iter_ops_work() {
    let path = PathTree::from_iter(vec![5, 6, 7, 8]);

    let path = path.append_segment(9);

    let mut i = 5;
    for val in &path {
        assert_eq!(i, *val);
        i += 1;
    }
}
