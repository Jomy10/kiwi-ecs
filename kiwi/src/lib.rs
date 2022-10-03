mod macros {
    pub use kiwi_macros::system;
    pub use kiwi_macros::query;
    pub use kiwi_macros::query_mut;
    pub use kiwi_macros::spawn_entity;
    pub use kiwi_macros::Component;
}
pub use macros::*;

mod world;
pub use world::World;

mod entity;

mod component;
pub use component::{ComponentId, Component};

mod archetype;
mod arch_store;
pub use arch_store::ArchetypeId;

mod arch {
    pub(crate) use crate::archetype::*;
    pub(crate) use crate::arch_store::*;
}

// impl std::fmt::Debug for ComponentColumn {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("ComponentColumn")
//             .field("components", &self.components.len())
//             .finish()
//     }
// }