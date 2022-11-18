//! See [GitHub](https://github.com/jomy10/kiwi-ecs) for detailed documentation.
// TODO: add docs to crate

mod macros {
    pub use kiwi_macros::system;
    pub use kiwi_macros::query;
    pub use kiwi_macros::query_mut;
    pub use kiwi_macros::spawn_entity;
    pub use kiwi_macros::Component;
    pub use kiwi_macros::flags;
}
pub use macros::*;

mod world;
pub use world::World;

mod entity;
pub use entity::EntityId;

mod component;
pub use component::{ComponentId, Component, Flag, FlagId};

mod archetype;
mod arch_store;
pub use arch_store::ArchetypeId;

mod arch {
    pub(crate) use crate::archetype::*;
    pub(crate) use crate::arch_store::*;
}
