use std::sync::RwLockReadGuard;

use crate::arch_store::ArchetypeId;

pub type ComponentId = u32;

pub trait Component {
    fn get_archetypes() -> RwLockReadGuard<'static, Vec<ArchetypeId>>; // RwLockReadGuard<'static, Arc<Vec<ArchetypeId>>> where Self: Sized;
    // fn get_archetypes() -> &'static Vec<ArchetypeId>;
    fn add_archetype(arch: ArchetypeId) where Self: Sized;
    fn id() -> ComponentId where Self: Sized;
}

/*
#[cfg(test)]
mod tests {
    use crate::*;
    use crate as kiwi_ecs;

    #[derive(Component)]
    struct SomeComponent;
    
    #[test]
    fn static_archetypes() {
        assert_eq!(*SomeComponent::get_archetypes(), Vec::<u32>::new());
        SomeComponent::add_archetype(5);
        assert_eq!(*SomeComponent::get_archetypes(), vec![5]);
    }
}
*/
