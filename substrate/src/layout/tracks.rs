//! Routing track management.

use geometry::span::Span;
use num::integer::{div_ceil, div_floor};
use serde::{Deserialize, Serialize};

/// A uniform set of tracks.
///
/// The track line and space must be even.
///
/// Track 0 is centered at `offset`.
/// Track 1 is centered at `offset + line + space`.
/// Track -1 is centered at `offset - (line + space)`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct UniformTracks {
    /// The width of each track.
    line: i64,
    /// Spacing between adjacent track edges.
    space: i64,
    /// An offset that translates all tracks.
    offset: i64,
}

impl UniformTracks {
    /// Create a uniform track set with the given line and space.
    pub fn new(line: i64, space: i64) -> Self {
        Self::with_offset(line, space, 0)
    }

    /// Create a uniform track set with the given line, space, and offset.
    pub fn with_offset(line: i64, space: i64, offset: i64) -> Self {
        assert_eq!(line & 1, 0, "track width must be even");
        assert_eq!(space & 1, 0, "track spacing must be even");
        assert!(line > 0);
        assert!(space > 0);
        Self {
            line,
            space,
            offset,
        }
    }

    /// Gets the coordinates of the `i`-th track.
    pub fn get(&self, idx: i64) -> Span {
        let start = self.offset + idx * self.pitch() - self.line / 2;
        Span::new(start, start + self.line)
    }

    /// The pitch (line + space) of the tracks.
    #[inline]
    pub fn pitch(&self) -> i64 {
        self.line + self.space
    }

    /// Iterates over a range of adjacent tracks.
    pub fn get_tracks(
        &self,
        range: impl Into<std::ops::Range<i64>>,
    ) -> impl Iterator<Item = Span> + '_ {
        range.into().map(|i| self.get(i))
    }

    /// Explicitly enumerates a range of adjacent tracks, returning an [`EnumeratedTracks`].
    ///
    /// Note that this uses `O(N)` storage, where `N` is the length of the range.
    pub fn enumerate(&self, range: impl Into<std::ops::Range<i64>>) -> EnumeratedTracks {
        self.get_tracks(range).collect()
    }

    /// Converts a geometric coordinate to the index of the nearest track.
    pub fn to_track_idx(&self, coord: i64, mode: RoundingMode) -> i64 {
        match mode {
            RoundingMode::Down => div_floor(coord - self.offset + self.line / 2, self.pitch()),
            RoundingMode::Up => div_ceil(coord - self.offset - self.line / 2, self.pitch()),
            RoundingMode::Nearest => div_floor(
                coord - self.offset + self.pitch() / 2 + self.line / 2,
                self.pitch(),
            ),
        }
    }
}

/// Rounding options.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Serialize, Deserialize)]
pub enum RoundingMode {
    /// Round to the nearest number.
    #[default]
    Nearest,
    /// Round down.
    Down,
    /// Round up.
    Up,
}

/// A set of explicitly listed, ordered tracks.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EnumeratedTracks {
    tracks: Vec<Span>,
}

impl EnumeratedTracks {
    /// Iterates over the tracks in this [`EnumeratedTracks`].
    pub fn tracks(&self) -> impl Iterator<Item = Span> + '_ {
        self.tracks.iter().copied()
    }

    /// Construct a new [`EnumeratedTracks`] from an iterator.
    ///
    /// # Panics
    ///
    /// Panics if the tracks are not in order.
    pub fn new(iter: impl IntoIterator<Item = Span>) -> Self {
        iter.into_iter().collect()
    }

    /// Returns the number of tracks in the set.
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    /// Returns `true` if the set of tracks is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }
}

impl FromIterator<Span> for EnumeratedTracks {
    fn from_iter<T: IntoIterator<Item = Span>>(iter: T) -> Self {
        let tracks: Vec<Span> = iter.into_iter().collect();
        // check that tracks are ordered and valid
        for (track, next) in tracks.iter().zip(tracks.iter().skip(1)) {
            assert!(next.start() > track.stop());
        }
        Self { tracks }
    }
}

impl IntoIterator for EnumeratedTracks {
    type Item = Span;
    type IntoIter = std::vec::IntoIter<Span>;
    fn into_iter(self) -> Self::IntoIter {
        self.tracks.into_iter()
    }
}

/// A set of tracks.
pub trait Tracks {
    /// The track at the given index.
    fn try_track(&self, idx: i64) -> Option<Span>;

    /// The range of valid indices, as a tuple `(min, max)`.
    ///
    /// If there is no min/max index, implementers should return `None`.
    fn try_range(&self) -> (Option<i64>, Option<i64>);

    /// The track at the given index, panicking if the index is out of bounds.
    #[inline]
    fn track(&self, idx: i64) -> Span {
        self.try_track(idx).expect("track index out of bounds")
    }
}

/// A finite set of tracks.
pub trait FiniteTracks {
    /// The range of valid indices, as a tuple `(min, max)`.
    ///
    /// The minimum acceptable index is `min`; the maximum acceptable index is `max-1`,
    /// which must be greater than or equal to `min`.
    fn range(&self) -> (i64, i64);
}

impl Tracks for EnumeratedTracks {
    fn try_track(&self, idx: i64) -> Option<Span> {
        let idx = usize::try_from(idx).ok()?;
        self.tracks.get(idx).copied()
    }

    fn try_range(&self) -> (Option<i64>, Option<i64>) {
        let range = <Self as FiniteTracks>::range(self);
        (Some(range.0), Some(range.1))
    }
}

impl FiniteTracks for EnumeratedTracks {
    fn range(&self) -> (i64, i64) {
        let max = i64::try_from(self.tracks.len()).expect("track list length is too long");
        (0, max)
    }
}

impl Tracks for UniformTracks {
    fn try_track(&self, idx: i64) -> Option<Span> {
        Some(self.get(idx))
    }

    fn try_range(&self) -> (Option<i64>, Option<i64>) {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use geometry::span::Span;

    use super::*;

    #[test]
    fn enumerated_tracks() {
        let tracks: EnumeratedTracks = [Span::new(10, 20), Span::new(30, 40), Span::new(80, 100)]
            .into_iter()
            .collect();

        assert_eq!(tracks.track(0), Span::new(10, 20));
        assert_eq!(tracks.track(1), Span::new(30, 40));
        assert_eq!(tracks.track(2), Span::new(80, 100));
        assert_eq!(tracks.range(), (0, 3));
        assert_eq!(tracks.try_track(-1), None);
        assert_eq!(tracks.try_track(3), None);
    }

    #[test]
    #[should_panic]
    fn enumerated_tracks_panics_when_tracks_are_out_of_order() {
        let _: EnumeratedTracks = [Span::new(10, 20), Span::new(15, 30), Span::new(80, 100)]
            .into_iter()
            .collect();
    }

    #[test]
    fn uniform_tracks() {
        let tracks = UniformTracks::new(20, 40);

        assert_eq!(tracks.track(-2), Span::new(-130, -110));
        assert_eq!(tracks.track(-1), Span::new(-70, -50));
        assert_eq!(tracks.track(0), Span::new(-10, 10));
        assert_eq!(tracks.track(1), Span::new(50, 70));
        assert_eq!(tracks.track(2), Span::new(110, 130));
        assert_eq!(tracks.try_range(), (None, None));
    }

    #[test]
    fn uniform_tracks_with_offset() {
        let tracks = UniformTracks::with_offset(20, 40, 15);

        assert_eq!(tracks.track(-2), Span::new(-115, -95));
        assert_eq!(tracks.track(-1), Span::new(-55, -35));
        assert_eq!(tracks.track(0), Span::new(5, 25));
        assert_eq!(tracks.track(1), Span::new(65, 85));
        assert_eq!(tracks.track(2), Span::new(125, 145));
        assert_eq!(tracks.try_range(), (None, None));
    }

    #[test]
    #[should_panic]
    fn uniform_tracks_requires_even_line_width() {
        UniformTracks::new(5, 40);
    }

    #[test]
    #[should_panic]
    fn uniform_tracks_requires_even_spacing() {
        UniformTracks::new(320, 645);
    }

    #[test]
    fn uniform_tracks_to_track_idx() {
        let tracks = UniformTracks::with_offset(260, 140, 130);
        assert_eq!(tracks.to_track_idx(-20, RoundingMode::Down), -1);
        assert_eq!(tracks.to_track_idx(-550, RoundingMode::Down), -2);
        assert_eq!(tracks.to_track_idx(-200, RoundingMode::Down), -1);
        assert_eq!(tracks.to_track_idx(-20, RoundingMode::Up), 0);
        assert_eq!(tracks.to_track_idx(-530, RoundingMode::Up), -1);
        assert_eq!(tracks.to_track_idx(-550, RoundingMode::Up), -2);
    }
}
