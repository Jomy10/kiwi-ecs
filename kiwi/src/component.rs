use std::sync::RwLockReadGuard;
use std::any::Any;

use crate::arch_store::ArchetypeId;

pub type ComponentId = u32;

pub trait Component {
    fn get_archetypes() -> RwLockReadGuard<'static, Vec<ArchetypeId>> where Self: Sized;
    fn add_archetype(arch: ArchetypeId) where Self: Sized;
    fn id() -> ComponentId where Self: Sized;
    fn as_any<'a>(&'a self) -> &'a dyn Any;
    fn as_any_mut<'a>(&'a mut self) -> &'a mut dyn Any;
}

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
