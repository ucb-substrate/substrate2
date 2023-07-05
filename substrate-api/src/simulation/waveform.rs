//! Time-domain waveforms.

use std::cmp::Ordering;
use std::iter::FusedIterator;

use serde::{Deserialize, Serialize};

/// A time-dependent waveform that owns its data.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Waveform {
    /// List of [`TimePoint`]s.
    values: Vec<TimePoint>,
}

/// A time-dependent waveform that references data stored elsewhere.
pub struct WaveformRef<'a> {
    t: &'a [f64],
    x: &'a [f64],
}

/// A single point `(t, x)` on a waveform.
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TimePoint {
    t: f64,
    x: f64,
}

impl TimePoint {
    /// Create a new [`TimePoint`].
    #[inline]
    pub fn new(t: f64, x: f64) -> Self {
        Self { t, x }
    }

    /// The time associated with this point.
    #[inline]
    pub fn t(&self) -> f64 {
        self.t
    }

    /// The value associated with this point.
    #[inline]
    pub fn x(&self) -> f64 {
        self.x
    }
}

impl From<(f64, f64)> for TimePoint {
    #[inline]
    fn from(value: (f64, f64)) -> Self {
        Self {
            t: value.0,
            x: value.1,
        }
    }
}

/// A time-domain waveform.
pub trait TimeWaveform {
    /// Get the value of the waveform at the given index.
    fn get(&self, idx: usize) -> Option<TimePoint>;

    /// Returns the number of time points in the waveform.
    fn len(&self) -> usize;

    /// Returns `true` if the waveform is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The time associated with the first point in the waveform.
    fn first_t(&self) -> Option<f64> {
        Some(self.first()?.t())
    }

    /// The value associated with the first point in the waveform.
    fn first_x(&self) -> Option<f64> {
        Some(self.first()?.x())
    }

    /// The time associated with the last point in the waveform.
    fn last_t(&self) -> Option<f64> {
        Some(self.last()?.t())
    }

    /// The value associated with the last point in the waveform.
    fn last_x(&self) -> Option<f64> {
        Some(self.last()?.x())
    }

    /// The first point in the waveform.
    fn first(&self) -> Option<TimePoint> {
        self.get(0)
    }

    /// The last point in the waveform.
    fn last(&self) -> Option<TimePoint> {
        self.get(self.len() - 1)
    }

    /// Returns an iterator over the edges in the waveform.
    ///
    /// See [`Edges`] for more information.
    fn edges(&self, threshold: f64) -> Edges<'_, Self> {
        Edges {
            waveform: self,
            idx: 0,
            thresh: threshold,
        }
    }

    /// Returns an iterator over the transitions in the waveform.
    ///
    /// See [`Transitions`] for more information.
    fn transitions(&self, low_threshold: f64, high_threshold: f64) -> Transitions<'_, Self> {
        assert!(high_threshold > low_threshold);
        Transitions {
            waveform: self,
            state: TransitionState::Unknown,
            t: 0.0,
            prev_idx: 0,
            idx: 0,
            low_thresh: low_threshold,
            high_thresh: high_threshold,
        }
    }

    /// Returns an iterator over the values in the waveform.
    ///
    /// See [`Values`] for more information.
    fn values(&self) -> Values<'_, Self> {
        Values {
            waveform: self,
            idx: 0,
        }
    }

    /// Returns the index of the last point in the waveform with a time before `t`.
    fn time_index_before(&self, t: f64) -> Option<usize> {
        search_for_time(self, t)
    }

    /// Retrieves the value of the waveform at the given time.
    ///
    /// By default, linearly interpolates between two adjacent points on the waveform.
    fn sample_at(&self, t: f64) -> f64 {
        let idx = self
            .time_index_before(t)
            .expect("cannot extrapolate to the requested time");
        debug_assert!(
            idx < self.len() - 1,
            "cannot extrapolate beyond end of signal"
        );
        let p0 = self.get(idx).unwrap();
        let p1 = self.get(idx + 1).unwrap();
        linear_interp(p0.t(), p0.x(), p1.t(), p1.x(), t)
    }

    /// Returns the maximum value seen in this waveform.
    fn max_x(&self) -> Option<f64> {
        let mut max = None;
        for i in 0..self.len() {
            let point = self.get(i)?;
            if let Some(max_val) = max.as_mut() {
                if *max_val < point.x {
                    *max_val = point.x;
                }
            } else {
                max = Some(point.x);
            }
        }
        max
    }

    /// Returns the minimum value seen in this waveform.
    fn min_x(&self) -> Option<f64> {
        let mut min = None;
        for i in 0..self.len() {
            let point = self.get(i)?;
            if let Some(min_val) = min.as_mut() {
                if *min_val > point.x {
                    *min_val = point.x;
                }
            } else {
                min = Some(point.x);
            }
        }
        min
    }

    /// Returns the middle value seen in this waveform.
    ///
    /// This is typically the arithmetic average of the max and min values.
    fn mid_x(&self) -> Option<f64> {
        Some((self.max_x()? + self.min_x()?) / 2.0)
    }

    /// Returns the time integral of this waveform.
    ///
    /// By default, uses trapezoidal integration.
    /// Returns 0.0 if the length of the waveform is less than 2.
    fn integral(&self) -> f64 {
        let n = self.len();
        if n < 2 {
            return 0.0;
        }

        let mut integral = 0.0;

        for i in 0..self.len() - 1 {
            let p0 = self.get(i).unwrap();
            let p1 = self.get(i + 1).unwrap();
            let dt = p1.t - p0.t;
            let avg = (p0.x + p1.x) / 2.0;
            integral += avg * dt;
        }

        integral
    }
}

fn linear_interp(t0: f64, y0: f64, t1: f64, y1: f64, t: f64) -> f64 {
    let c = (t - t0) / (t1 - t0);
    y0 + c * (y1 - y0)
}

/// An iterator over the values in the waveform.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct Values<'a, T: ?Sized> {
    waveform: &'a T,
    idx: usize,
}

impl<'a, T> Iterator for Values<'a, T>
where
    T: TimeWaveform,
{
    type Item = TimePoint;
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.waveform.get(self.idx);
        if val.is_some() {
            self.idx += 1;
        }
        val
    }
}
impl<'a, T> FusedIterator for Values<'a, T> where T: TimeWaveform {}

impl TimeWaveform for Waveform {
    fn get(&self, idx: usize) -> Option<TimePoint> {
        self.values.get(idx).copied()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a> TimeWaveform for WaveformRef<'a> {
    fn get(&self, idx: usize) -> Option<TimePoint> {
        if idx >= self.len() {
            return None;
        }
        Some(TimePoint::new(self.t[idx], self.x[idx]))
    }

    fn len(&self) -> usize {
        self.t.len()
    }
}

/// Possible edge directions.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum EdgeDir {
    /// A falling edge.
    Falling,
    /// A rising edge.
    Rising,
}

/// An edge.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Edge {
    pub(crate) t: f64,
    pub(crate) start_idx: usize,
    pub(crate) dir: EdgeDir,
}

/// An iterator over the edges in a waveform.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct Edges<'a, T: ?Sized> {
    waveform: &'a T,
    idx: usize,
    thresh: f64,
}

#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
enum TransitionState {
    /// High at the given time.
    High,
    /// Unknown.
    #[default]
    Unknown,
    /// Low at the given time.
    Low,
}

/// An iterator over the transitions in a waveform.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct Transitions<'a, T: ?Sized> {
    waveform: &'a T,
    state: TransitionState,
    /// Time at which the waveform was in either a high or low state.
    t: f64,
    prev_idx: usize,
    /// Index of the **next** element to process.
    idx: usize,
    low_thresh: f64,
    high_thresh: f64,
}

/// A single observed transition in a waveform.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Transition {
    pub(crate) start_t: f64,
    pub(crate) end_t: f64,
    pub(crate) start_idx: usize,
    pub(crate) end_idx: usize,
    pub(crate) dir: EdgeDir,
}

impl Waveform {
    /// Creates a new, empty waveform.
    #[inline]
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Creats a new waveform with a single point `(0.0, x)`.
    pub fn with_initial_value(x: f64) -> Self {
        Self {
            values: vec![TimePoint::new(0.0, x)],
        }
    }

    /// Adds the given point to the waveform.
    pub fn push(&mut self, t: f64, x: f64) {
        if let Some(tp) = self.last_t() {
            assert!(t > tp);
        }
        self.values.push(TimePoint::new(t, x));
    }
}

impl FromIterator<(f64, f64)> for Waveform {
    fn from_iter<T: IntoIterator<Item = (f64, f64)>>(iter: T) -> Self {
        Self {
            values: iter
                .into_iter()
                .map(|(t, x)| TimePoint::new(t, x))
                .collect(),
        }
    }
}

pub(crate) fn edge_crossing_time(t0: f64, y0: f64, t1: f64, y1: f64, thresh: f64) -> f64 {
    let c = (thresh - y0) / (y1 - y0);
    debug_assert!(c >= 0.0);
    debug_assert!(c <= 1.0);
    t0 + c * (t1 - t0)
}

impl<'a, T> Edges<'a, T>
where
    T: TimeWaveform,
{
    fn check(&mut self) -> Option<Edge> {
        let p0 = self.waveform.get(self.idx)?;
        let p1 = self.waveform.get(self.idx + 1)?;
        let first = p0.x - self.thresh;
        let second = p1.x - self.thresh;
        if first.signum() != second.signum() {
            let dir = if second.signum() > 0.0 {
                EdgeDir::Rising
            } else {
                EdgeDir::Falling
            };
            Some(Edge {
                dir,
                t: edge_crossing_time(p0.t, p0.x, p1.t, p1.x, self.thresh),
                start_idx: self.idx,
            })
        } else {
            None
        }
    }
}

impl<'a, T> Iterator for Edges<'a, T>
where
    T: TimeWaveform,
{
    type Item = Edge;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.waveform.len() - 1 {
            return None;
        }
        loop {
            let val = self.check();
            self.idx += 1;
            if val.is_some() {
                break val;
            }
            if self.idx >= self.waveform.len() - 1 {
                break None;
            }
        }
    }
}
impl<'a, T> FusedIterator for Edges<'a, T> where T: TimeWaveform {}

impl<'a, T> Transitions<'a, T>
where
    T: TimeWaveform,
{
    fn check(&mut self) -> Option<(TransitionState, f64)> {
        let pt = self.waveform.get(self.idx)?;
        Some((
            if pt.x >= self.high_thresh {
                TransitionState::High
            } else if pt.x <= self.low_thresh {
                TransitionState::Low
            } else {
                TransitionState::Unknown
            },
            pt.t,
        ))
    }
}

impl<'a, T> Iterator for Transitions<'a, T>
where
    T: TimeWaveform,
{
    type Item = Transition;
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.waveform.len() - 1 {
            return None;
        }
        loop {
            use TransitionState::*;

            let (val, t) = self.check()?;
            let end_idx = self.idx;
            self.idx += 1;

            match (self.state, val) {
                (High, Low) => {
                    self.state = Low;
                    let (old_t, old_idx) = (self.t, self.prev_idx);
                    self.prev_idx = end_idx;
                    self.t = t;
                    return Some(Transition {
                        start_t: old_t,
                        end_t: t,
                        start_idx: old_idx,
                        end_idx,
                        dir: EdgeDir::Falling,
                    });
                }
                (Low, High) => {
                    self.state = High;
                    let (old_t, old_idx) = (self.t, self.prev_idx);
                    self.prev_idx = end_idx;
                    self.t = t;
                    return Some(Transition {
                        start_t: old_t,
                        end_t: t,
                        start_idx: old_idx,
                        end_idx,
                        dir: EdgeDir::Rising,
                    });
                }
                (Unknown, High) => {
                    self.state = High;
                    self.t = t;
                    self.prev_idx = end_idx;
                }
                (Unknown, Low) => {
                    self.state = Low;
                    self.t = t;
                    self.prev_idx = end_idx;
                }
                (High, High) | (Low, Low) => {
                    self.t = t;
                    self.prev_idx = end_idx;
                }
                _ => (),
            }
        }
    }
}

impl<'a, T> FusedIterator for Transitions<'a, T> where T: TimeWaveform {}

impl Default for Waveform {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Index<usize> for Waveform {
    type Output = TimePoint;
    fn index(&self, index: usize) -> &Self::Output {
        self.values.index(index)
    }
}

impl std::ops::IndexMut<usize> for Waveform {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.values.index_mut(index)
    }
}

impl EdgeDir {
    /// Returns `true` if this is a rising edge.
    #[inline]
    pub fn is_rising(&self) -> bool {
        matches!(self, EdgeDir::Rising)
    }

    /// Returns `true` if this is a falling edge.
    #[inline]
    pub fn is_falling(&self) -> bool {
        matches!(self, EdgeDir::Falling)
    }
}

impl Edge {
    /// The direction (rising or falling) of the edge.
    #[inline]
    pub fn dir(&self) -> EdgeDir {
        self.dir
    }

    /// The time at which the waveform crossed the threshold.
    ///
    /// The waveform is linearly interpolated to find the threshold crossing time.
    #[inline]
    pub fn t(&self) -> f64 {
        self.t
    }

    /// The index in the waveform **before** the threshold was passed.
    #[inline]
    pub fn idx_before(&self) -> usize {
        self.start_idx
    }

    /// The index in the waveform **after** the threshold was passed.
    #[inline]
    pub fn idx_after(&self) -> usize {
        self.start_idx + 1
    }
}

impl Transition {
    /// The direction (rising or falling) of the transition.
    #[inline]
    pub fn dir(&self) -> EdgeDir {
        self.dir
    }

    /// The time at which this transition starts.
    #[inline]
    pub fn start_time(&self) -> f64 {
        self.start_t
    }

    /// The time at which this transition ends.
    #[inline]
    pub fn end_time(&self) -> f64 {
        self.end_t
    }

    /// The index of the transition start point in the original waveform.
    #[inline]
    pub fn start_idx(&self) -> usize {
        self.start_idx
    }

    /// The index of the transition end point in the original waveform.
    #[inline]
    pub fn end_idx(&self) -> usize {
        self.end_idx
    }

    /// The duration of the transition.
    ///
    /// Equal to the difference between the end time and the start time.
    #[inline]
    pub fn duration(&self) -> f64 {
        self.end_time() - self.start_time()
    }

    /// The average of the start and end times.
    #[inline]
    pub fn center_time(&self) -> f64 {
        (self.start_time() + self.end_time()) / 2.0
    }
}

impl<'a> WaveformRef<'a> {
    /// Creates a new waveform referencing the given `t` and `x` data.
    ///
    /// # Panics
    ///
    /// Panics if the two slices have different lengths.
    #[inline]
    pub fn new(t: &'a [f64], x: &'a [f64]) -> Self {
        assert_eq!(t.len(), x.len());
        Self { t, x }
    }
}

fn search_for_time<T>(data: &T, target: f64) -> Option<usize>
where
    T: TimeWaveform + ?Sized,
{
    if data.is_empty() {
        return None;
    }

    let mut ans = None;
    let mut lo = 0usize;
    let mut hi = data.len() - 1;
    let mut x;
    while lo < hi {
        let mid = (lo + hi) / 2;
        x = data.get(mid).unwrap().t();
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

/// Parameters for constructing a [`DigitalWaveformBuilder`].
pub struct DigitalWaveformParams {
    /// The digital supply voltage (V).
    pub vdd: f64,
    /// The digital clock period (sec).
    pub period: f64,
    /// The rise time (sec).
    pub tr: f64,
    /// The fall time (sec).
    pub tf: f64,
}

/// A builder for creating clocked digital waveforms.
pub struct DigitalWaveformBuilder {
    params: DigitalWaveformParams,

    ctr: usize,
    values: Vec<TimePoint>,
    state: TransitionState,
}

impl DigitalWaveformBuilder {
    /// Creates a new builder with the given parameters.
    pub fn new(params: impl Into<DigitalWaveformParams>) -> Self {
        Self {
            params: params.into(),
            values: Vec::new(),
            ctr: 0,
            state: TransitionState::Unknown,
        }
    }

    /// Adds one cycle of logical high to the waveform.
    ///
    /// If the waveform was previously logical low, the waveform will
    /// transition to logical high with a duration governed by the rise time parameter.
    pub fn add_hi(&mut self) -> &mut Self {
        let cycle = self.ctr as f64;
        let cycle_next = (self.ctr + 1) as f64;
        match self.state {
            TransitionState::High => {}
            TransitionState::Low => {
                self.values.push(TimePoint::new(
                    cycle * self.params.period + self.params.tr,
                    self.params.vdd,
                ));
            }
            TransitionState::Unknown => {
                assert_eq!(self.ctr, 0);
                self.values.push(TimePoint::new(0f64, self.params.vdd));
            }
        }
        self.values.push(TimePoint::new(
            cycle_next * self.params.period,
            self.params.vdd,
        ));

        self.ctr += 1;
        self.state = TransitionState::High;
        self
    }

    /// Adds one cycle of logical low to the waveform.
    ///
    /// If the waveform was previously logical high, the waveform will
    /// transition to logical low with a duration governed by the fall time parameter.
    pub fn add_lo(&mut self) -> &mut Self {
        let cycle = self.ctr as f64;
        let cycle_next = (self.ctr + 1) as f64;
        match self.state {
            TransitionState::High => {
                self.values.push(TimePoint::new(
                    cycle * self.params.period + self.params.tf,
                    0f64,
                ));
            }
            TransitionState::Low => {}
            TransitionState::Unknown => {
                assert_eq!(self.ctr, 0);
                self.values.push(TimePoint::new(0f64, 0f64));
            }
        }
        self.values
            .push(TimePoint::new(cycle_next * self.params.period, 0f64));

        self.ctr += 1;
        self.state = TransitionState::Low;
        self
    }

    /// Adds one cycle of the given bit value to the waveform.
    ///
    /// If `bit` is `true`, this is equivalent to calling [`DigitalWaveformBuilder::add_hi`].
    /// If `bit` is `false`, this is equivalent to calling [`DigitalWaveformBuilder::add_lo`].
    #[inline]
    pub fn add(&mut self, bit: bool) -> &mut Self {
        if bit {
            self.add_hi()
        } else {
            self.add_lo()
        }
    }

    /// Consumes the builder, producing a [`Waveform`].
    pub fn build(self) -> Waveform {
        Waveform {
            values: self.values,
        }
    }
}

#[cfg(test)]
mod tests {
    use approx::{assert_relative_eq, assert_relative_ne};

    use super::*;

    #[test]
    fn waveform_edges() {
        let wav =
            Waveform::from_iter([(0., 0.), (1., 1.), (2., 0.9), (3., 0.1), (4., 0.), (5., 1.)]);
        let edges = wav.edges(0.5).collect::<Vec<_>>();
        assert_eq!(
            edges,
            vec![
                Edge {
                    t: 0.5,
                    start_idx: 0,
                    dir: EdgeDir::Rising,
                },
                Edge {
                    t: 2.5,
                    start_idx: 2,
                    dir: EdgeDir::Falling,
                },
                Edge {
                    t: 4.5,
                    start_idx: 4,
                    dir: EdgeDir::Rising,
                }
            ]
        );
    }

    #[test]
    fn waveform_transitions() {
        let wav =
            Waveform::from_iter([(0., 0.), (1., 1.), (2., 0.9), (3., 0.1), (4., 0.), (5., 1.)]);
        let transitions = wav.transitions(0.1, 0.9).collect::<Vec<_>>();
        assert_eq!(
            transitions,
            vec![
                Transition {
                    start_t: 0.,
                    start_idx: 0,
                    end_t: 1.,
                    end_idx: 1,
                    dir: EdgeDir::Rising,
                },
                Transition {
                    start_t: 2.,
                    start_idx: 2,
                    end_t: 3.,
                    end_idx: 3,
                    dir: EdgeDir::Falling,
                },
                Transition {
                    start_t: 4.,
                    start_idx: 4,
                    end_t: 5.,
                    end_idx: 5,
                    dir: EdgeDir::Rising,
                },
            ]
        );
    }

    #[test]
    fn waveform_integral() {
        let wav = Waveform::from_iter([
            (0., 0.),
            (1., 1.),
            (2., 0.9),
            (3., 0.1),
            (4., 0.),
            (5., 1.),
            (8., 1.1),
        ]);
        let expected = 0.5 + 0.95 + 0.5 + 0.05 + 0.5 + 3.0 * 1.05;
        let integral = wav.integral();
        assert_relative_eq!(integral, expected);
        assert_relative_ne!(integral, expected + 1e-12);
        assert_relative_ne!(integral, expected - 1e-12);
    }
}
