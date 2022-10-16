use crate::entity::{EntityStore, EntityId};
use crate::arch::{ArchStore, ArchetypeId, NewEntityResult};
use crate::component::Component;

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
        // let ent = self.entity_store.entities[ent_id as usize];
        self.entity_store.kill(ent_id);
        self.arch_store.remove_entity(ent_id);
    }
    
    pub fn entity_count(&self) -> usize {
        self.entity_store.entity_count()
    }

    /// Returns the component of type `T` for entity with id `entity`.
    pub fn get_component<T: Component + 'static>(&self, entity: EntityId) -> &T {
        let entity = &self.entity_store.entities()[entity as usize];
        // let comp = self.arch_store.archetypes[entity.arch_id as usize].get_component::<T>(entity.arch_row);
        let comp = self.arch_store.get_archetype(entity.arch_id).get_component::<T>(entity.arch_row);
        let dyn_comp = &**unsafe { comp.assume_init_ref() }; // The user always needs to specify the components for the entity
        let comp: Option<&T> = dyn_comp.as_any().downcast_ref::<T>();
        let comp: &T = unsafe { comp.unwrap_unchecked() };
        comp
    }
    
    pub fn set_component<T: Component + 'static>(&mut self, entity: EntityId, comp: T) {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype_mut(entity.arch_id).set_component(entity.arch_row, comp);
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
