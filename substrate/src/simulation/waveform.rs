//! Time-domain waveforms.

use std::cmp::Ordering;
use std::iter::FusedIterator;
use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

/// A time-dependent waveform that owns its data.
#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Hash, Eq, Serialize, Deserialize)]
pub struct Waveform<T> {
    /// List of [`TimePoint`]s.
    values: Vec<TimePoint<T>>,
}

/// A time-dependent waveform that references data stored elsewhere.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Hash, Eq)]
pub struct WaveformRef<'a, T> {
    t: &'a [T],
    x: &'a [T],
}

/// A single point `(t, x)` on a waveform.
#[derive(
    Debug, Default, Copy, Clone, PartialEq, Hash, Ord, Eq, PartialOrd, Serialize, Deserialize,
)]
pub struct TimePoint<T> {
    t: T,
    x: T,
}

impl<T> TimePoint<T> {
    /// Create a new [`TimePoint`].
    #[inline]
    pub fn new(t: T, x: T) -> Self {
        Self { t, x }
    }

    /// Converts the TimePoint's datatype into another datatype.
    pub fn convert_into<U>(self) -> TimePoint<U>
    where
        T: Into<U>,
    {
        TimePoint {
            t: self.t.into(),
            x: self.x.into(),
        }
    }
}

impl<T> TimePoint<T>
where
    T: Copy,
{
    /// The time associated with this point.
    #[inline]
    pub fn t(&self) -> T {
        self.t
    }

    /// The value associated with this point.
    #[inline]
    pub fn x(&self) -> T {
        self.x
    }
}

impl<T> From<(T, T)> for TimePoint<T> {
    #[inline]
    fn from(value: (T, T)) -> Self {
        Self {
            t: value.0,
            x: value.1,
        }
    }
}

/// A time-domain waveform.
pub trait TimeWaveform {
    /// The datatype of time and signal values in the waveform.
    ///
    /// Typically, this should be [`f64`] or [`rust_decimal::Decimal`].
    type Data: Copy
        + From<i32>
        + Add<Self::Data, Output = Self::Data>
        + Div<Self::Data, Output = Self::Data>
        + PartialOrd
        + Sub<Self::Data, Output = Self::Data>
        + Mul<Self::Data, Output = Self::Data>;
    /// Get the value of the waveform at the given index.
    fn get(&self, idx: usize) -> Option<TimePoint<Self::Data>>;

    /// Returns the number of time points in the waveform.
    fn len(&self) -> usize;

    /// Returns `true` if the waveform is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The time associated with the first point in the waveform.
    fn first_t(&self) -> Option<Self::Data> {
        Some(self.first()?.t())
    }

    /// The value associated with the first point in the waveform.
    fn first_x(&self) -> Option<Self::Data> {
        Some(self.first()?.x())
    }

    /// The time associated with the last point in the waveform.
    fn last_t(&self) -> Option<Self::Data> {
        Some(self.last()?.t())
    }

    /// The value associated with the last point in the waveform.
    fn last_x(&self) -> Option<Self::Data> {
        Some(self.last()?.x())
    }

    /// The first point in the waveform.
    fn first(&self) -> Option<TimePoint<Self::Data>> {
        self.get(0)
    }

    /// The last point in the waveform.
    fn last(&self) -> Option<TimePoint<Self::Data>> {
        self.get(self.len() - 1)
    }

    /// Returns an iterator over the edges in the waveform.
    ///
    /// See [`Edges`] for more information.
    fn edges(&self, threshold: Self::Data) -> Edges<'_, Self, Self::Data> {
        Edges {
            waveform: self,
            idx: 0,
            thresh: threshold,
        }
    }

    /// Returns an iterator over the transitions in the waveform.
    ///
    /// See [`Transitions`] for more information.
    fn transitions(
        &self,
        low_threshold: Self::Data,
        high_threshold: Self::Data,
    ) -> Transitions<'_, Self, Self::Data> {
        assert!(high_threshold > low_threshold);
        Transitions {
            waveform: self,
            state: TransitionState::Unknown,
            t: Self::Data::from(0),
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
    fn time_index_before(&self, t: Self::Data) -> Option<usize> {
        search_for_time(self, t)
    }

    /// Retrieves the value of the waveform at the given time.
    ///
    /// By default, linearly interpolates between two adjacent points on the waveform.
    fn sample_at(&self, t: Self::Data) -> Self::Data {
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
    fn max_x(&self) -> Option<Self::Data> {
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
    fn min_x(&self) -> Option<Self::Data> {
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
    fn mid_x(&self) -> Option<Self::Data> {
        Some((self.max_x()? + self.min_x()?) / Self::Data::from(2))
    }

    /// Returns the time integral of this waveform.
    ///
    /// By default, uses trapezoidal integration.
    /// Returns 0.0 if the length of the waveform is less than 2.
    fn integral(&self) -> Self::Data {
        let n = self.len();
        if n < 2 {
            return Self::Data::from(0);
        }

        let mut integral = Self::Data::from(0);

        for i in 0..self.len() - 1 {
            let p0 = self.get(i).unwrap();
            let p1 = self.get(i + 1).unwrap();
            let dt = p1.t - p0.t;
            let avg = (p0.x + p1.x) / Self::Data::from(2);
            integral = integral + avg * dt;
        }

        integral
    }
}

fn linear_interp<T>(t0: T, y0: T, t1: T, y1: T, t: T) -> T
where
    T: Copy
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialOrd
        + Sub<T, Output = T>
        + Mul<T, Output = T>,
{
    let c = (t - t0) / (t1 - t0);
    y0 + c * (y1 - y0)
}

/// An iterator over the values in the waveform.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct Values<'a, T: ?Sized> {
    waveform: &'a T,
    idx: usize,
}

impl<'a, W> Iterator for Values<'a, W>
where
    W: TimeWaveform,
{
    type Item = TimePoint<W::Data>;
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.waveform.get(self.idx);
        if val.is_some() {
            self.idx += 1;
        }
        val
    }
}

impl<'a, W> FusedIterator for Values<'a, W> where W: TimeWaveform {}

impl<T> TimeWaveform for Waveform<T>
where
    T: Copy
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialOrd
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + From<i32>,
{
    type Data = T;

    fn get(&self, idx: usize) -> Option<TimePoint<T>> {
        self.values.get(idx).copied()
    }

    fn len(&self) -> usize {
        self.values.len()
    }
}

impl<'a, T> TimeWaveform for WaveformRef<'a, T>
where
    T: Copy
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialOrd
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + From<i32>,
{
    type Data = T;
    fn get(&self, idx: usize) -> Option<TimePoint<T>> {
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
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
pub struct Edge<T> {
    pub(crate) t: T,
    pub(crate) start_idx: usize,
    pub(crate) dir: EdgeDir,
}

/// An iterator over the edges in a waveform.
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
pub struct Edges<'a, W: ?Sized, T> {
    waveform: &'a W,
    idx: usize,
    thresh: T,
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
pub struct Transitions<'a, W: ?Sized, T> {
    waveform: &'a W,
    state: TransitionState,
    /// Time at which the waveform was in either a high or low state.
    t: T,
    prev_idx: usize,
    /// Index of the **next** element to process.
    idx: usize,
    low_thresh: T,
    high_thresh: T,
}

/// A single observed transition in a waveform.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Transition<T> {
    pub(crate) start_t: T,
    pub(crate) end_t: T,
    pub(crate) start_idx: usize,
    pub(crate) end_idx: usize,
    pub(crate) dir: EdgeDir,
}

impl<T> Waveform<T> {
    /// Creates a new, empty waveform.
    #[inline]
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Converts the waveform's datatype into another datatype.
    pub fn convert_into<U>(self) -> Waveform<U>
    where
        T: Into<U>,
    {
        let values = self
            .values
            .into_iter()
            .map(|tp| tp.convert_into())
            .collect();
        Waveform { values }
    }

    /// Creates a new waveform with a single point `(0, x)`.
    pub fn with_initial_value(x: T) -> Self
    where
        T: From<i32>,
    {
        Self {
            values: vec![TimePoint::new(T::from(0), x)],
        }
    }

    /// Adds the given point to the waveform.
    pub fn push(&mut self, t: T, x: T)
    where
        Self: TimeWaveform<Data = T>,
        T: PartialOrd,
    {
        if let Some(tp) = self.last_t() {
            assert!(t > tp);
        }
        self.values.push(TimePoint::new(t, x));
    }
}

impl<T> FromIterator<(T, T)> for Waveform<T> {
    fn from_iter<I: IntoIterator<Item = (T, T)>>(iter: I) -> Self {
        Self {
            values: iter
                .into_iter()
                .map(|(t, x)| TimePoint::new(t, x))
                .collect(),
        }
    }
}

pub(crate) fn edge_crossing_time<T>(t0: T, y0: T, t1: T, y1: T, thresh: T) -> T
where
    T: Copy
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialOrd
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + From<i32>,
{
    let c = (thresh - y0) / (y1 - y0);
    debug_assert!(c >= T::from(0));
    debug_assert!(c <= T::from(1));
    t0 + c * (t1 - t0)
}

impl<'a, W> Edges<'a, W, W::Data>
where
    W: TimeWaveform,
    <W as TimeWaveform>::Data: Copy,
{
    fn check(&mut self) -> Option<Edge<W::Data>> {
        let p0 = self.waveform.get(self.idx)?;
        let p1 = self.waveform.get(self.idx + 1)?;
        let first = p0.x - self.thresh;
        let second = p1.x - self.thresh;
        if (first >= <W as TimeWaveform>::Data::from(0))
            != (second >= <W as TimeWaveform>::Data::from(0))
        {
            let dir = if second >= <W as TimeWaveform>::Data::from(0) {
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

impl<'a, W> Iterator for Edges<'a, W, W::Data>
where
    W: TimeWaveform,
{
    type Item = Edge<W::Data>;
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

impl<'a, W> FusedIterator for Edges<'a, W, W::Data> where W: TimeWaveform {}

impl<'a, W> Transitions<'a, W, <W as TimeWaveform>::Data>
where
    W: TimeWaveform,
    W::Data: PartialOrd,
{
    fn check(&mut self) -> Option<(TransitionState, W::Data)> {
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

impl<'a, W> Iterator for Transitions<'a, W, <W as TimeWaveform>::Data>
where
    W: TimeWaveform,
    W::Data: PartialOrd,
{
    type Item = Transition<W::Data>;
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

impl<'a, W> FusedIterator for Transitions<'a, W, <W as TimeWaveform>::Data>
where
    W: TimeWaveform,
    <W as TimeWaveform>::Data: Copy
        + Add<<W as TimeWaveform>::Data, Output = <W as TimeWaveform>::Data>
        + Div<<W as TimeWaveform>::Data, Output = <W as TimeWaveform>::Data>
        + PartialOrd
        + Sub<<W as TimeWaveform>::Data, Output = <W as TimeWaveform>::Data>
        + Mul<<W as TimeWaveform>::Data, Output = <W as TimeWaveform>::Data>
        + From<i32>,
{
}

impl<T> Default for Waveform<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::ops::Index<usize> for Waveform<T> {
    type Output = TimePoint<T>;
    fn index(&self, index: usize) -> &Self::Output {
        self.values.index(index)
    }
}

impl<T> std::ops::IndexMut<usize> for Waveform<T> {
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

impl<T> Edge<T> {
    /// The direction (rising or falling) of the edge.
    #[inline]
    pub fn dir(&self) -> EdgeDir {
        self.dir
    }

    /// The time at which the waveform crossed the threshold.
    ///
    /// The waveform is linearly interpolated to find the threshold crossing time.
    #[inline]
    pub fn t(&self) -> T
    where
        T: Copy,
    {
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

impl<T> Transition<T> {
    /// The direction (rising or falling) of the transition.
    #[inline]
    pub fn dir(&self) -> EdgeDir {
        self.dir
    }

    /// The time at which this transition starts.
    #[inline]
    pub fn start_time(&self) -> T
    where
        T: Copy,
    {
        self.start_t
    }

    /// The time at which this transition ends.
    #[inline]
    pub fn end_time(&self) -> T
    where
        T: Copy,
    {
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
    pub fn duration(&self) -> T
    where
        T: Copy + Sub<T, Output = T>,
    {
        self.end_time() - self.start_time()
    }

    /// The average of the start and end times.
    #[inline]
    pub fn center_time(&self) -> T
    where
        T: Copy + Add<T, Output = T> + Div<T, Output = T> + From<i32>,
    {
        (self.start_time() + self.end_time()) / T::from(2)
    }
}

impl<'a, T> WaveformRef<'a, T> {
    /// Creates a new waveform referencing the given `t` and `x` data.
    ///
    /// # Panics
    ///
    /// Panics if the two slices have different lengths.
    #[inline]
    pub fn new(t: &'a [T], x: &'a [T]) -> Self {
        assert_eq!(t.len(), x.len());
        Self { t, x }
    }
}

fn search_for_time<W>(data: &W, target: <W as TimeWaveform>::Data) -> Option<usize>
where
    W: TimeWaveform + ?Sized,
    <W as TimeWaveform>::Data: PartialOrd,
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
        match target.partial_cmp(&x)? {
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
pub struct DigitalWaveformParams<T> {
    /// The digital supply voltage (V).
    pub vdd: T,
    /// The digital clock period (sec).
    pub period: T,
    /// The rise time (sec).
    pub tr: T,
    /// The fall time (sec).
    pub tf: T,
}

/// A builder for creating clocked digital waveforms.
pub struct DigitalWaveformBuilder<T> {
    params: DigitalWaveformParams<T>,

    ctr: usize,
    values: Vec<TimePoint<T>>,
    state: TransitionState,
}

impl<T> DigitalWaveformBuilder<T>
where
    T: Copy
        + Add<T, Output = T>
        + Div<T, Output = T>
        + PartialOrd
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + From<i32>,
{
    /// Creates a new builder with the given parameters.
    pub fn new(params: impl Into<DigitalWaveformParams<T>>) -> Self {
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
        let cycle = T::from(self.ctr as i32);
        let cycle_next = T::from((self.ctr + 1) as i32);
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
                self.values
                    .push(TimePoint::new(T::from(0), self.params.vdd));
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
        let cycle = T::from(self.ctr as i32);
        let cycle_next = T::from((self.ctr + 1) as i32);
        match self.state {
            TransitionState::High => {
                self.values.push(TimePoint::new(
                    cycle * self.params.period + self.params.tf,
                    T::from(0),
                ));
            }
            TransitionState::Low => {}
            TransitionState::Unknown => {
                assert_eq!(self.ctr, 0);
                self.values.push(TimePoint::new(T::from(0), T::from(0)));
            }
        }
        self.values
            .push(TimePoint::new(cycle_next * self.params.period, T::from(0)));

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
    pub fn build(self) -> Waveform<T> {
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
                },
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
