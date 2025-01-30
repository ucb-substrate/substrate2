use std::cmp::Ordering;

pub mod analysis;
pub mod ascii;
pub mod binary;

#[cfg(test)]
mod tests;

pub type Result<T> = anyhow::Result<T>;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub(crate) fn bin_search_before(data: &[f64], target: f64) -> Option<usize> {
    if data.is_empty() {
        return None;
    }

    let mut ans = None;
    let mut lo = 0usize;
    let mut hi = data.len() - 1;
    let mut x;
    while lo < hi {
        let mid = (lo + hi) / 2;
        x = data[mid];
        match target.total_cmp(&x) {
            Ordering::Less => hi = mid - 1,
            Ordering::Greater => {
                lo = mid + 1;
                ans = Some(mid)
            }
            Ordering::Equal => return Some(mid),
        }
    }

    ans
}
