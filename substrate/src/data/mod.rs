use std::marker::PhantomData;

type InvariantPhantom<T> = PhantomData<*mut T>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id<'ctx>(u32, InvariantPhantom<&'ctx ()>);

impl<'ctx> Id<'ctx> {
    pub(crate) fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }
}

impl<'ctx> std::fmt::Debug for Id<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({:?})", self.0)
    }
}

pub trait DataType: Clone {
    type Instantiated<'ctx>: InstantiatedData<'ctx>;

    fn flatten(&self) -> Vec<Signal<DynamicDirection>>;

    fn instantiate<'ctx, 'id>(
        &self,
        ids: &'id [Id<'ctx>],
    ) -> (Self::Instantiated<'ctx>, &'id [Id<'ctx>]);
}

pub trait InstantiatedData<'ctx>
where
    Self::Type: DataType<Instantiated<'ctx> = Self>,
{
    type Type: DataType;

    fn type_of(&self) -> Self::Type;

    fn flatten(&self) -> Vec<InstantiatedSignal<'ctx, DynamicDirection>>;

    fn as_type<Other: DataType>(&self, other: &Other) -> Other::Instantiated<'ctx> {
        let flat: Vec<_> = self.flatten().into_iter().map(|s| s.id).collect();
        let (inst, rest) = other.instantiate(&flat);
        assert!(rest.is_empty());
        inst
    }

    fn as_type_of<Other: InstantiatedData<'ctx>>(&self, other: &Other) -> Other {
        self.as_type(&other.type_of())
    }
}

// FIXME: seal
pub trait DirectionType: Clone {
    type Flipped: DirectionType;

    fn direction(&self) -> Direction;
    fn flip(&self) -> Self::Flipped;

    fn to_dynamic(&self) -> DynamicDirection {
        DynamicDirection::new(self.direction())
    }
}

pub trait FixedDirection: DirectionType {
    fn new() -> Self;
}
macro_rules! declare_directions {
    ($($id:ident => $flipped:ident),+ $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum Direction {
            $($id),+
        }

        impl Direction {
            pub fn flip(&self) -> Self {
                use Direction::*;
                match self {
                    $($id => $flipped),+
                }
            }
        }

        declare_directions!(%structs,$($id => $flipped),+);
    };
    (%structs,$id:ident => $flipped:ident,$($rest_id:ident => $rest_flipped:ident),+) => {
        declare_directions!(%structs,$id => $flipped);
        declare_directions!(%structs,$($rest_id => $rest_flipped),+);
    };
    (%structs,$id:ident => $flipped:ident) => {
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
        pub struct $id;
        impl DirectionType for $id {
            type Flipped = $flipped;

            fn direction(&self) -> Direction { Direction::$id }
            fn flip(&self) -> Self::Flipped { $flipped }
        }
        impl FixedDirection for $id {
            fn new() -> Self { $id }
        }
    };
}
declare_directions! {
    InOut => InOut,
    Input => Output,
    Output => Input,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicDirection {
    direction: Direction,
}
impl DynamicDirection {
    fn new(direction: Direction) -> Self {
        Self { direction }
    }
}
impl DirectionType for DynamicDirection {
    type Flipped = DynamicDirection;

    fn direction(&self) -> Direction {
        self.direction
    }

    fn flip(&self) -> Self::Flipped {
        Self {
            direction: self.direction.flip(),
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Signal<DIR: DirectionType> {
    direction: DIR,
}

impl<DIR: FixedDirection> Signal<DIR> {
    pub fn new() -> Self {
        Signal {
            direction: DIR::new(),
        }
    }
}

impl<DIR: DirectionType> Signal<DIR> {
    pub fn from_direction(direction: DIR) -> Self {
        Signal { direction }
    }
}

impl<DIR: DirectionType> DataType for Signal<DIR> {
    type Instantiated<'ctx> = InstantiatedSignal<'ctx, DIR>;

    fn flatten(&self) -> Vec<Signal<DynamicDirection>> {
        vec![self.to_dynamic()]
    }

    fn instantiate<'ctx, 'id>(
        &self,
        ids: &'id [Id<'ctx>],
    ) -> (Self::Instantiated<'ctx>, &'id [Id<'ctx>]) {
        if let [id, rest @ ..] = ids {
            (InstantiatedSignal::new(*id, self.direction.clone()), rest)
        } else {
            // This shouldn't happen if the traits are implemented correctly,
            // and isn't something you'd reasonably want to recover from.
            // FIXME: better message
            panic!("bad instantiate")
        }
    }
}

impl<DIR: DirectionType> Signal<DIR> {
    fn flip(&self) -> Signal<DIR::Flipped> {
        Signal {
            direction: self.direction.flip(),
        }
    }

    fn to_dynamic(&self) -> Signal<DynamicDirection> {
        Signal {
            direction: self.direction.to_dynamic(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstantiatedSignal<'ctx, DIR> {
    id: Id<'ctx>,
    direction: DIR,
}

impl<'ctx, DIR: DirectionType> InstantiatedSignal<'ctx, DIR> {
    fn new(id: Id<'ctx>, direction: DIR) -> Self {
        Self { id, direction }
    }

    fn to_dynamic(&self) -> InstantiatedSignal<'ctx, DynamicDirection> {
        InstantiatedSignal::new(self.id, self.direction.to_dynamic())
    }
}

impl<'ctx, DIR: DirectionType> InstantiatedData<'ctx> for InstantiatedSignal<'ctx, DIR> {
    fn flatten(&self) -> Vec<InstantiatedSignal<'ctx, DynamicDirection>> {
        vec![self.to_dynamic()]
    }

    type Type = Signal<DIR>;

    fn type_of(&self) -> Self::Type {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct Flipped<D: DataType> {
    ty: D,
}
impl<D: DataType> Flipped<D> {
    pub fn new(ty: D) -> Self {
        Self { ty }
    }
}
impl<D: DataType> DataType for Flipped<D> {
    type Instantiated<'ctx> = D::Instantiated<'ctx>;

    fn flatten(&self) -> Vec<Signal<DynamicDirection>> {
        self.ty.flatten().iter().map(|s| s.flip()).collect()
    }

    fn instantiate<'ctx, 'id>(
        &self,
        ids: &'id [Id<'ctx>],
    ) -> (Self::Instantiated<'ctx>, &'id [Id<'ctx>]) {
        self.ty.instantiate(ids)
    }
}
// FIXME: direction for Flipped::Instantiated::Type
