use type_dispatch_macros::{dispatch_const, impl_dispatch};

trait ApplyFilter<M> {
    fn apply(medium: &mut M);
}

struct GaussianBlur;
struct Fisheye;
struct Pixelate;

#[derive(Default)]
struct Painting {
    strokes: Vec<u64>,
}
#[derive(Default)]
struct Photoshop {
    pixels: Vec<Vec<u64>>,
}
#[derive(Default)]
struct Illustrator {
    vectors: Vec<(u64, u64)>,
}

// Implement individual filters manually as their implementations vary widely.
impl ApplyFilter<Painting> for GaussianBlur {
    fn apply(medium: &mut Painting) {
        medium.strokes.iter_mut().for_each(|stroke| *stroke *= 2);
    }
}
impl ApplyFilter<Photoshop> for GaussianBlur {
    fn apply(medium: &mut Photoshop) {
        medium
            .pixels
            .iter_mut()
            .for_each(|pixels| pixels.iter_mut().for_each(|pixel| *pixel *= 2));
    }
}
impl ApplyFilter<Illustrator> for GaussianBlur {
    fn apply(medium: &mut Illustrator) {
        medium
            .vectors
            .iter_mut()
            .for_each(|vector| *vector = (2 * vector.0, 2 * vector.1));
    }
}
impl ApplyFilter<Painting> for Fisheye {
    fn apply(medium: &mut Painting) {
        medium.strokes.iter_mut().for_each(|stroke| *stroke += 2);
    }
}
impl ApplyFilter<Photoshop> for Fisheye {
    fn apply(medium: &mut Photoshop) {
        medium
            .pixels
            .iter_mut()
            .for_each(|pixels| pixels.iter_mut().for_each(|pixel| *pixel += 2));
    }
}
impl ApplyFilter<Illustrator> for Fisheye {
    fn apply(medium: &mut Illustrator) {
        medium
            .vectors
            .iter_mut()
            .for_each(|vector| *vector = (vector.0 + 2, vector.1 + 2));
    }
}
impl ApplyFilter<Photoshop> for Pixelate {
    fn apply(medium: &mut Photoshop) {
        medium
            .pixels
            .iter_mut()
            .for_each(|pixels| pixels.iter_mut().for_each(|pixel| *pixel /= 2));
    }
}

struct StarryNight;

impl ApplyFilter<Painting> for StarryNight {
    fn apply(medium: &mut Painting) {
        medium.strokes.extend(vec![1, 2, 3, 4, 5]);
    }
}
impl ApplyFilter<Photoshop> for StarryNight {
    fn apply(medium: &mut Photoshop) {
        medium.pixels.extend(vec![vec![1, 2], vec![3, 4]]);
    }
}
impl ApplyFilter<Illustrator> for StarryNight {
    fn apply(medium: &mut Illustrator) {
        medium.vectors.extend(vec![(1, 2), (3, 4)]);
    }
}

struct RetroStarryNight;

// Use `impl_dispatch` to reuse similar code between two "media".
#[impl_dispatch({Painting; Illustrator})]
impl<M> ApplyFilter<M> for RetroStarryNight {
    fn apply(medium: &mut M) {
        StarryNight::apply(medium);
        Fisheye::apply(medium);
        for _ in 0..dispatch_const!(
            match M {
                Painting => 2: u64,
                Illustrator => 3: usize,
            }
        ) {
            GaussianBlur::apply(medium);
        }
    }
}
impl ApplyFilter<Photoshop> for RetroStarryNight {
    fn apply(medium: &mut Photoshop) {
        StarryNight::apply(medium);
        Pixelate::apply(medium);
    }
}

#[test]
fn impl_dispatch_works() {
    let mut painting = Painting::default();
    let mut photoshop = Photoshop::default();
    let mut illustrator = Illustrator::default();

    RetroStarryNight::apply(&mut painting);
    RetroStarryNight::apply(&mut photoshop);
    RetroStarryNight::apply(&mut illustrator);

    assert_eq!(painting.strokes, vec![12, 16, 20, 24, 28]);
    assert_eq!(photoshop.pixels, vec![vec![0, 1], vec![1, 2]]);
    assert_eq!(illustrator.vectors, vec![(24, 32), (40, 48)]);
}
