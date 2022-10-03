use std::mem::MaybeUninit;
use std::collections::{HashMap, HashSet};

use crate::component::{Component, ComponentId};
use crate::entity::{EntityId, EntityStore};

// macro_rules! get_entity_ids_iter {
//     ($arch:tt, $ent_store:tt) => {
//         $arch.entities.iter()
//             .map(|v| *v)
//             .filter(|id| $ent_store.is_alive(*id))
//     }
// }

macro_rules! get_entity_ids_enumerate_iter {
    ($arch:tt, $ent_store:tt) => {
        $arch.entities.iter()
            .enumerate()
            .filter(|(_, id)| $ent_store.is_alive(**id))
    }
}

pub(crate) type ArchRowId = u32;

struct ComponentColumn {
    components: Vec<MaybeUninit<Box<dyn Component>>>,
}

impl ComponentColumn {
    fn new() -> Self {
        Self {
            components: Vec::new()
        }
    }
}

pub(crate) struct Archetype {
    components: HashMap<ComponentId, ComponentColumn>,
    available_ent_ids: HashSet<EntityId>, // TODO: vec
    // /// The size of the components arrays in `self.components`
    // ent_count: u32,
    /// Index in this vector is the same as in `components.components`
    /// It holds entity ids that belong to this archetype, but they might be
    /// dead, or moved to another archetype. i.e. do alive check
    entities: Vec<EntityId>,
}

impl Archetype {
    pub(crate) fn new(components: &Vec<ComponentId>) -> Self {
        // let comps: Vec<ComponentColumn> = Vec::with_capacity(components.len());
        let mut comps = HashMap::with_capacity(components.len());
        for component in components {
            comps.insert(*component, ComponentColumn::new());
        }
        
        Self {
            components: comps,
            available_ent_ids: HashSet::new(),
            // ent_count: 0,
            entities: Vec::new()
        }
    }
    
    #[inline]
    /// Get an empty entity id
    pub(crate) fn new_archrow_id(&mut self, entity: EntityId) -> ArchRowId {
        if self.available_ent_ids.len() != 0 {
            let id = *unsafe { self.available_ent_ids.iter().next().unwrap_unchecked() };
            self.available_ent_ids.remove(&id);
            self.entities[id as usize] = entity;
            return id;
        } else {
            let id = self.entities.len();
            self.entities.push(entity);
            return id as ArchRowId;
        }
    }

    #[inline]
    pub(crate) fn set_component<T: Component + 'static>(&mut self, entity_id: ArchRowId, component: T) {
        let component_col = self.components.get_mut(&T::id()).unwrap();
        if component_col.components.len() <= entity_id as usize {
            component_col.components.resize_with((entity_id as usize) + 1, MaybeUninit::uninit);
        }
        component_col.components[entity_id as usize] = MaybeUninit::new(Box::new(component));
    }
    
    #[inline]
    /// Get component of type `T` for entity with arch row `entity_id`
    pub(crate) fn get_component<T: Component>(&self, entity_id: ArchRowId) -> &MaybeUninit<Box<dyn Component>> {
        let component_col = self.components.get(&T::id()).unwrap();
        &component_col.components[entity_id as usize]
    }
    
    #[inline]
    pub(crate) fn get_all_components<T: Component + 'static>(&self, ent_ids: &Vec<ArchRowId>) -> Vec<&T> {
        let component_col: &ComponentColumn = self.components.get(&T::id()).unwrap();
        ent_ids.iter()
            .map(|ent_id| {
                dbg!(&component_col.components);
                dbg!(ent_id);
                let dyn_comp = &**unsafe { component_col.components[*ent_id as usize].assume_init_ref() };
                let comp: Option<&T> = dyn_comp.as_any().downcast_ref::<T>();
                unsafe { comp.unwrap_unchecked() }
            }).collect()
    }
    
    #[inline]
    /// Get all components of type `T` for the entities with ids `ent_ids`
    pub(crate) fn get_all_components_mut<T: Component + 'static>(&mut self, ent_ids: &Vec<EntityId>) -> Vec<*mut T> {
        let component_col: &mut ComponentColumn = self.components.get_mut(&T::id()).unwrap();
        ent_ids.iter()
            .map(|ent_id| {
                let dyn_comp = &mut**unsafe { component_col.components[*ent_id as usize].assume_init_mut() };
                let comp = dyn_comp.as_any_mut().downcast_mut::<T>();
                let comp = unsafe { comp.unwrap_unchecked() };
                comp as *mut T
            })
            .collect::<Vec<*mut T>>()
    }
    
    // #[inline]
    // fn get_entity_ids(&self, ent_store: &EntityStore) -> Vec<EntityId>{
    //     get_entity_ids_iter!(self, ent_store).collect()
    // }
    
    #[inline]
    pub(crate) fn get_arch_rows(&self, ent_store: &EntityStore) -> Vec<ArchRowId> {
        get_entity_ids_enumerate_iter!(self, ent_store)
            .map(|(row, _)| row as u32)
            .collect()
    }
    
    #[inline]
    pub(crate) fn get_rows_and_ids(&self, ent_store: &EntityStore) -> Vec<(ArchRowId, EntityId)> {
        get_entity_ids_enumerate_iter!(self, ent_store)
            .map(|(row, id)| (row as u32, *id))
            .collect()
    }
}
