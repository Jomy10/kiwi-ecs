use std::sync::RwLockReadGuard;

use crate::arch_store::ArchetypeId;

pub type ComponentId = u32;

pub trait Component {
    /// Gets all the archetypes that have this component. Only used internally
    fn get_archetypes() -> RwLockReadGuard<'static, Vec<ArchetypeId>>; // RwLockReadGuard<'static, Arc<Vec<ArchetypeId>>> where Self: Sized;
    /// Add an archetype to this component. Only used internally
    fn add_archetype(arch: ArchetypeId) where Self: Sized;
    /// The id of this component. Only used internally
    fn id() -> ComponentId where Self: Sized;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use crate as kiwi_ecs;

    #[derive(Component)]
    struct SomeComponent(u8);
    
    #[test]
    fn static_archetypes() {
        assert_eq!(*SomeComponent::get_archetypes(), Vec::<u32>::new());
        SomeComponent::add_archetype(5);
        assert_eq!(*SomeComponent::get_archetypes(), vec![5]);
    }
}

pub type FlagId = u32;

pub trait Flag: std::convert::Into<FlagId> {}
