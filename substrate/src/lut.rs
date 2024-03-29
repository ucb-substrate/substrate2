//! Lookup tables.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use splines::{Key, Spline};

/// A 1D lookup table.
// TODO verify that length of keys and values match, and that k1 is sorted
#[allow(missing_docs)]
#[derive(Debug, Default, Clone, Eq, PartialEq, Builder, Serialize, Deserialize)]
#[builder(pattern = "owned")]
pub struct Lut1<K1, V> {
    k1: Vec<K1>,
    values: Vec<V>,
}

/// A 2D lookup table.
#[allow(missing_docs)]
#[derive(Debug, Default, Clone, Eq, PartialEq, Builder, Serialize, Deserialize)]
#[builder(pattern = "owned")]
pub struct Lut2<K1, K2, V> {
    k1: Vec<K1>,
    k2: Vec<K2>,
    // row major order
    values: Vec<Vec<V>>,
}

/// Extrapolation options.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub enum Extrapolation {
    /// Do not extrapolate.
    #[default]
    None,
    /// Rounds up to the first key in the lookup table's range.
    RoundUp,
}

impl<K1, K2, V> Lut2<K1, K2, V> {
    /// Create a new [`Lut2Builder`].
    pub fn builder() -> Lut2Builder<K1, K2, V> {
        Default::default()
    }
}

impl<K1, K2, V> Lut2<K1, K2, V>
where
    K1: Ord,
    K2: Ord,
{
    /// Lookup a value for the given keys.
    pub fn get(&self, k1: &K1, k2: &K2) -> Option<&V> {
        let i1 = self.k1.partition_point(|k| k < k1);
        let i2 = self.k2.partition_point(|k| k < k2);
        if k1 < self.k1.first()? || k2 < self.k2.first()? {
            return None;
        }
        self.values.get(i1)?.get(i2)
    }
}

impl FloatLut2 {
    /// Lookup a value for the given keys, interpolating as necessary.
    pub fn getf(&self, k1: f64, k2: f64) -> Option<f64> {
        let interp1 = (0..self.k1.len())
            .map(|i| {
                Spline::from_vec(
                    self.k2
                        .iter()
                        .copied()
                        .zip(self.values.get(i)?.iter().copied())
                        .map(|(k, v)| Key::new(k, v, splines::Interpolation::Linear))
                        .collect(),
                )
                .sample(k2)
            })
            .collect::<Option<Vec<f64>>>()?;

        Spline::from_vec(
            self.k1
                .iter()
                .copied()
                .zip(interp1)
                .map(|(k, v)| Key::new(k, v, splines::Interpolation::Linear))
                .collect(),
        )
        .sample(k1)
    }

    /// Lookup a value for the given keys, interpolating as necessary.
    ///
    /// Can extrapolate beyond the bounds of the key ranges.
    pub fn getf_extrapolate(
        &self,
        mut k1: f64,
        mut k2: f64,
        extrapolate: Extrapolation,
    ) -> Option<f64> {
        if extrapolate == Extrapolation::RoundUp {
            (k1, k2) = (k1.max(*self.k1.first()?), k2.max(*self.k2.first()?));
        }

        self.getf(k1, k2)
    }
}

/// A floating point 1D LUT.
pub type FloatLut1 = Lut1<f64, f64>;

/// A floating point 2D LUT.
pub type FloatLut2 = Lut2<f64, f64, f64>;

#[cfg(test)]
mod tests {
    use float_eq::float_eq;

    use super::*;

    #[test]
    fn test_lut_u64() {
        let lut = Lut2::<u64, u64, u64>::builder()
            .k1(vec![5, 6, 7])
            .k2(vec![1, 2, 3])
            .values(vec![vec![1, 5, 9], vec![2, 4, 8], vec![3, 6, 7]])
            .build()
            .unwrap();

        assert_eq!(lut.get(&5, &2), Some(&5));
        assert_eq!(lut.get(&4, &2), None);
        assert_eq!(lut.get(&7, &3), Some(&7));
        assert_eq!(lut.get(&8, &3), None);
        assert_eq!(lut.get(&6, &4), None);
        assert_eq!(lut.get(&6, &0), None);
    }

    #[test]
    fn test_lut_f64() {
        let lut = FloatLut2::builder()
            .k1(vec![5., 6., 7.])
            .k2(vec![1., 2., 3.])
            .values(vec![vec![1., 5., 9.], vec![2., 4., 8.], vec![3., 6., 7.]])
            .build()
            .unwrap();

        assert!(float_eq!(lut.getf(5., 2.).unwrap(), 5., r2nd <= 1e-8));
        assert!(float_eq!(lut.getf(5., 2.5).unwrap(), 7., r2nd <= 1e-8));
        assert!(float_eq!(lut.getf(6.5, 1.5).unwrap(), 3.75, r2nd <= 1e-8));
        assert_eq!(lut.getf(4.5, 2.5), None);
    }
}
