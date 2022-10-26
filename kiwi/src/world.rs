use crate::entity::{EntityStore, EntityId};
use crate::arch::{ArchStore, NewEntityResult};
use crate::component::Component;

// TODO: remove pub
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
    ///
    /// # Safety
    /// No checks are performed wheter the component is a flag (empty struct).
    ///
    /// To check whether the entity has a flag component, use the `has_component`
    /// function defined on `World`.
    pub unsafe fn get_component<T: Component + 'static>(&self, entity: EntityId) -> &T {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype(entity.arch_id).get_component::<T>(entity.arch_row)
    }
    
    /// Returns a mutable referencce to the component of type `T` for entity with id `entity`
    ///
    /// # Safety
    /// No checks are performed wheter the component is a flag (empty struct).
    ///
    /// To check whether the entity has a flag component, use the `has_component`
    /// function defined on `World`.
    pub unsafe fn get_component_mut<T: Component + 'static>(&mut self, entity: EntityId) -> &mut T {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype_mut(entity.arch_id).get_component_mut::<T>(entity.arch_row)
    }
    
    /// Set an entity's component.
    ///
    /// # Panics
    /// if the component does not exist for the given entity
    pub fn set_component<T: Component + 'static>(&mut self, entity: EntityId, comp: T) {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype_mut(entity.arch_id).set_component(entity.arch_row, comp);
    }
    
    /// Check whether an enttity contains the given component
    pub fn has_component<C: Component>(&self, entity: EntityId) -> bool {
        let entity = &self.entity_store.entities()[entity as usize];
        self.arch_store.get_archetype(entity.arch_id).has_component(C::id())
    }
    
    /// Returns whether the component of type `C` is a flag (unit struct)
    pub fn is_flag<C: Component>(&self) -> bool {
        return std::mem::size_of::<C>() == 0;
    }

    pub fn temp_query1<'a, A: Component + 'static>(&'a self) -> impl std::iter::Iterator<Item = &'a A> + 'a {
        let archetypes_a = A::get_archetypes();
        
        (*archetypes_a).clone().into_iter()
            .flat_map(|arch_id| {
                let archetype = self.arch_store.get_archetype(arch_id);
                let entities: Vec<crate::arch::ArchRowId> = archetype.get_arch_rows(&self.entity_store).collect();

                unsafe { archetype.get_all_components::<A>(entities) }
            })
    }

    pub fn temp_query
        <
            'a,
            A: Component + 'static,
            B: Component + 'static,
        >
        (&'a self)
        -> impl std::iter::Iterator<Item = (&'a A, &'a B)> + 'a
    {
        let archetypes_a = A::get_archetypes();
        let archetypes_b = B::get_archetypes();
        
        (*archetypes_a).clone().into_iter()
            .filter(move |elem| archetypes_b.contains(elem))
            .flat_map(|arch_id| {
                let archetype = self.arch_store.get_archetype(arch_id);
                let entities: Vec<crate::arch::ArchRowId> = archetype.get_arch_rows(&self.entity_store).collect();
                
                std::iter::zip(
                    unsafe { archetype.get_all_components::<A>(entities.clone()) },
                    unsafe { archetype.get_all_components::<B>(entities) }
                )
            })
    }
    
    pub fn temp_query_mut<
        'a,
        A: Component + 'static,
        B: Component + 'static,
    >(&'a mut self) -> impl std::iter::Iterator<Item = (&'a mut A, &'a mut B)> + 'a
    {
        let archetypes_a = A::get_archetypes();
        let archetypes_b = B::get_archetypes();
        
        (*archetypes_a).clone().into_iter()
            .filter(move |elem| archetypes_b.contains(elem))
            .flat_map(|arch_id| {
                let archetype: *mut crate::arch::Archetype = self.arch_store.get_archetype_mut(arch_id);
                let entities: Vec<crate::arch::ArchRowId> = unsafe { (*archetype).get_arch_rows(&self.entity_store).collect() };
                
                std::iter::zip(
                    unsafe { (*archetype).get_all_components_mut::<A>(entities.clone()) },
                    unsafe { (*archetype).get_all_components_mut::<B>(entities.clone()) }
                )
            })
    }

    pub fn temp_query_mut_id<
        'a,
        A: Component + 'static,
    >(&'a mut self) -> impl std::iter::Iterator<Item = (EntityId, &'a mut A)> + 'a
    {
        let archetypes_a = A::get_archetypes();
        
        (*archetypes_a).clone().into_iter()
            .flat_map(|arch_id| {
                let archetype: *mut crate::arch::Archetype = self.arch_store.get_archetype_mut(arch_id);
                let entities: Vec<crate::arch::ArchRowId> = unsafe { (*archetype).get_arch_rows(&self.entity_store).collect() };
                
                std::iter::zip(
                    entities.into_iter(),
                    unsafe { (*archetype).get_all_components_mut::<A>(entities.clone()) },
                )
            })
    }
    
    pub fn temp_query3<
        'a,
        A: Component + 'static,
        B: Component + 'static,
        C: Component + 'static,
    > (&'a self) -> impl std::iter::Iterator<Item = (&'a A, &'a B, &'a C)> + 'a
    {
        let archetypes_a = A::get_archetypes();
        let archetypes_b = B::get_archetypes();
        let archetypes_c = C::get_archetypes();
        
        archetypes_a.clone().into_iter()
            .filter(move |elem| archetypes_b.contains(elem))
            .filter(move |elem| archetypes_c.contains(elem))
            .flat_map(|arch_id| {
                let archetype = self.arch_store.get_archetype(arch_id);
                let entities: Vec<crate::arch::ArchRowId> = archetype.get_arch_rows(&self.entity_store).collect();
                std::iter::zip(
                    unsafe { archetype.get_all_components::<A>(entities.clone()) },
                    std::iter::zip(
                        unsafe { archetype.get_all_components::<B>(entities.clone()) },
                        unsafe { archetype.get_all_components::<C>(entities) },
                    )
                )
            }).map(|tuple| (tuple.0, tuple.1.0, tuple.1.1))
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
