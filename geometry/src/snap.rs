//! Snapping utilities (eg. snap to a grid).

/// Snaps `pos` to the nearest multiple of `grid`.
pub const fn snap_to_grid(pos: i64, grid: i64) -> i64 {
    assert!(grid > 0);

    let rem = pos.rem_euclid(grid);
    assert!(rem >= 0);
    assert!(rem < grid);
    if rem <= grid / 2 {
        pos - rem
    } else {
        pos + grid - rem
    }
}
