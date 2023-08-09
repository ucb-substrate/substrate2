# Type Dispatch

Utilities for dispatching based on generic types.

# Usage

```rust
struct GenericStruct<A, B>(A, B);

// Creates 4 trait implementations.
#[impl_dispatch({u64; u16}, {u32, usize; u8, u64})]
impl<A, B, C> Into<C> for GenericStruct<A, B> {
    fn into(self) -> C {
       self.0 as C + self.1 as C + dispatch_type!(
            match A, B {
                u64, u32 => 0..self.1,
                u64, u8 => vec![self.0 + self.1 as u64],
                u16, u32 => "ABC".to_string(),
                u16, u8 => "ABCD",
            }           
        ).len() as C
   }
}

let x: usize = GenericStruct(1u64, 3u32).into();
assert_eq!(x, 7);
let x: u64 = GenericStruct(1u64, 3u8).into();
assert_eq!(x, 5);
let x: usize = GenericStruct(1u16, 3u32).into();
assert_eq!(x, 7);
let x: u64 = GenericStruct(1u16, 3u8).into();
assert_eq!(x, 8);
```

```rust
#[derive(Debug, Default, PartialEq, Eq)]
struct Painting(Vec<usize>);
struct Stroke {
    thickness: usize,
}
impl Painting {
    fn draw(&mut self, stroke: Stroke) {
        self.0.push(2 * stroke.thickness)
    }
}
#[derive(Debug, Default, PartialEq, Eq)]
struct Photoshop(Vec<usize>);
struct Filter {
    strength: usize,
}
impl Photoshop {
    fn draw(&mut self, filter: Filter) {
        self.0.push(5 * filter.strength)
    }
}

struct Dispatcher<T>(usize, PhantomData<T>);

impl Dispatch for Dispatcher<Painting> {
    type Output = Stroke;
    fn dispatch(self) -> Self::Output {
        Stroke { thickness: self.0 }
    }
}
impl Dispatch for Dispatcher<Photoshop> {
    type Output = Filter;
    fn dispatch(self) -> Self::Output {
        Filter { strength: self.0 }
    }
}

#[impl_dispatch({Painting; Photoshop})]
impl<A> Into<A> for Vec<usize> {
    fn into(self) -> A {
        let mut drawing = A::default();
        for num in self {
            drawing.draw(Dispatcher(num, PhantomData::<A>).dispatch());
        }
        drawing
    }
}

let painting: Painting = vec![1, 2, 3].into();
let photoshop: Photoshop = vec![1, 2, 3].into();
assert_eq!(painting, Painting(vec![2, 4, 6]));
assert_eq!(photoshop, Photoshop(vec![5, 10, 15]));
```
