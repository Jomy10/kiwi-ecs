use crate::entity::{EntityStore, EntityId};
use crate::arch::{ArchStore, NewEntityResult};
use crate::component::{Component, Flag};

pub struct World {
    entity_store: EntityStore,
    arch_store: ArchStore,
}

impl World {
    pub fn new() -> Self {
        Self {
            entity_store: EntityStore::new(),
            arch_store: ArchStore::new(),
        }
    }

    kiwi_internal_macros::gen_spawn_entity!();

    /// Kills an entity
    pub fn kill(&mut self, ent_id: EntityId) {
        let ent = &self.entity_store.entities()[ent_id as usize];
        self.arch_store.remove_entity(ent);

        self.entity_store.kill(ent_id);
    }
    
    pub fn is_alive(&mut self, ent_id: EntityId) -> bool {
        self.entity_store.is_alive(ent_id)
    }
    
    pub fn entity_count(&self) -> usize {
        self.entity_store.entity_count()
    }

    // TODO: get component builder for an entity
    // world.get_components(entity_id) // returns (&World, &Entity)
    //    .component::<T>()
    //    .component::<T>()
    //    .get();
    
    /// Returns the component of type `T` for entity with id `entity`.
    ///
    /// # Panics
    /// if the component does not exist for the given entity
    pub fn get_component<T: Component + 'static>(&self, entity: EntityId) -> &T {
        let entity = &self.entity_store.entities()[entity as usize];
        unsafe { self.arch_store.get_archetype(entity.arch_id).get_component::<T>(entity.arch_row) }
    }
    
    /// Returns a mutable referencce to the component of type `T` for entity with id `entity`
    ///
    /// # Panics
    /// if the component does not exist for the given entity
    pub fn get_component_mut<T: Component + 'static>(&mut self, entity: EntityId) -> &mut T {
        let entity = &self.entity_store.entities()[entity as usize];
        unsafe { self.arch_store.get_archetype_mut(entity.arch_id).get_component_mut::<T>(entity.arch_row) }
    }
    
    /// Set an entity's component.
    ///
    /// # Panics
    /// if the component does not exist for the given entity
    pub fn set_component<T: Component + 'static>(&mut self, entity: EntityId, comp: T) {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype_mut(entity.arch_id).set_component(entity.arch_row, comp);
    }
    
    /// Check whether an entity contains the given component
    pub fn has_component<C: Component>(&self, entity: EntityId) -> bool {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype(entity.arch_id).has_component(C::id())
    }
    
    pub fn has_flag<F: Flag>(&self, entity: EntityId, flag: F) -> bool {
        self.entity_store.has_flag(entity, flag.into())
    }
    
    pub fn set_flag<F: Flag>(&mut self, entity: EntityId, flag: F) {
        self.entity_store.set_flag(entity, flag.into())
    }
}

// Queries
impl World {
    pub fn query_ids0(&self) -> Vec<EntityId> {
        self.entity_store.entities().iter()
            .enumerate()
            .map(|(id, _)| id as EntityId)
            .filter(|id| {
            self.entity_store.is_alive(*id)
        }).collect()
    }
    
    kiwi_internal_macros::gen_query!();
}
