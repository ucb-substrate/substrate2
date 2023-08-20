//! Routing track management.

use geometry::span::Span;
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
