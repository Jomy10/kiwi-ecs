use crate::entity::{EntityStore, EntityId};
use crate::arch::{ArchStore, NewEntityResult};
use crate::component::{Component, Flag};

/// The `World` is the entry point to an ecs
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
    ///
    /// This means the `EntityId` will be reused for other entities. This also
    /// implies that `world.is_alive(the_killed_enity_id)` will not be accurate.
    /// If you want to keep an id killed, use `world.kill_and_keep(id)`. You should
    /// still free those ids using `world.free_id` later so that the memory can be reused.
    pub fn kill(&mut self, ent_id: EntityId) {
        let ent = &self.entity_store.entities()[ent_id as usize];
        self.arch_store.remove_entity(ent);

        self.entity_store.kill(ent_id);
    }
    
    /// Kills an entity without telling the ecs to reuse its id
    ///
    /// This means that `world.is_alive` can be used accurately.
    ///
    /// You should use `world.free_id(the_killed_entity_id)` later to reuse the memory used by the
    /// killed entity (64 bits per entity).
    pub fn kill_and_keep(&mut self, ent_id: EntityId) {
        let ent = &self.entity_store.entities()[ent_id as usize];
        self.arch_store.remove_entity(ent);
        
        self.entity_store.kill(ent_id);
    }
    
    /// Frees an entity id, meaning that the `world.is_alive` method will no
    /// longer be accurate for this entity.
    ///
    /// Memory used to store this entity id (64 bits) will be reused.
    pub fn free_id(&mut self, ent_id: EntityId) {
        self.entity_store.free_id(ent_id);
    }
    
    /// Check whether an entity is alive. This can only be used accurately when
    /// `world.kill_and_keep` is used instead of `world.kill`.
    pub fn is_alive(&mut self, ent_id: EntityId) -> bool {
        self.entity_store.is_alive(ent_id)
    }
    
    /// Returns the amount of entities that are alive
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
    
    /// Returns whether the entity has the specified flag set.
    pub fn has_flag<F: Flag>(&self, entity: EntityId, flag: F) -> bool {
        self.entity_store.has_flag(entity, flag.into())
    }
    
    /// Sets a flag for an entity
    ///
    /// Always use the same enum for flags. Don't mix flag enums, because the
    /// id of the enum variant is used to determine whether the entity has
    /// a specific flag.
    pub fn set_flag<F: Flag>(&mut self, entity: EntityId, flag: F) {
        self.entity_store.set_flag(entity, flag.into())
    }

    /// Remove a flag from an entity
    pub fn unset_flag<F: Flag>(&mut self, entity: EntityId, flag: F) {
        self.entity_store.unset_flag(entity, flag.into())
    }
}

// Queries
impl World {
    /// Query all entity ids
    pub fn query_ids<'a>(&'a self) -> impl std::iter::Iterator<Item = EntityId> + 'a {
        self.entity_store.entities().iter()
            .enumerate()
            .map(|(id, _)| id as EntityId)
            .filter(|id| {
                self.entity_store.is_alive(*id)
            })
    }
    
    #[inline]
    #[doc(hidden)]
    pub fn query_ids0<'a>(&'a self) -> impl std::iter::Iterator<Item = EntityId> + 'a {
        self.query_ids()
    }
    
    kiwi_internal_macros::gen_query!();
}
