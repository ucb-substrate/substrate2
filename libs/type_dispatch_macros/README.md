# Type Dispatch

Macros for dispatching based on generic types.

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
