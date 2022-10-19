use std::mem::MaybeUninit;
use std::collections::HashMap;

use crate::component::{Component, ComponentId};
use crate::entity::{EntityId, EntityStore};

macro_rules! get_entity_ids_enumerate_iter {
    ($arch:tt, $ent_store:tt) => {
        $arch.entities.iter()
            .enumerate()
            .filter(|(_, id)| $ent_store.is_alive(**id))
    }
}

pub(crate) type ArchRowId = u32;

struct ComponentColumn {
    // components: Vec<MaybeUninit<Box<dyn Component>>>,
    components: Vec<MaybeUninit<u8>>,
    size: usize
}

impl ComponentColumn {
    fn new(size: usize) -> Self {
        Self {
            components: Vec::new(),
            size
        }
    }
}

pub(crate) struct Archetype {
    components: HashMap<ComponentId, ComponentColumn>,
    available_ent_ids: Vec<ArchRowId>,
    entities: Vec<EntityId>,
}

impl Archetype {
    pub(crate) fn new(components: &Vec<ComponentId>, sizes: &Vec<usize>) -> Self {
        // let comps: Vec<ComponentColumn> = Vec::with_capacity(components.len());
        let mut comps = HashMap::with_capacity(components.len());
        // for component in components {
        for i in 0..components.len() {
            comps.insert(components[i], ComponentColumn::new(sizes[i]));
        }
        
        Self {
            components: comps,
            available_ent_ids: Vec::new(),
            entities: Vec::new()
        }
    }
    
    #[inline]
    /// Get an empty entity id
    pub(crate) fn new_archrow_id(&mut self, entity: EntityId) -> ArchRowId {
        if let Some(id) = self.available_ent_ids.pop() {
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
        let component_col = self.components.get_mut(&T::id())
            .expect(&format!("Component {} does not exist for entity with id {}", std::any::type_name::<T>(), entity_id));
        // should makes the function safe + check above
        if component_col.components.len() <= entity_id as usize * component_col.size {
            component_col.components.resize_with((entity_id as usize) * component_col.size + component_col.size, MaybeUninit::uninit);
        }
        let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
        let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
        unsafe { *(comps_ptr.offset(entity_id as isize)) = MaybeUninit::new(component); }
    }
    
    // TODO: get_component_mut
    
    #[inline]
    /// Get component of type `T` for entity with arch row `entity_id`
    pub(crate) fn get_component<T: Component>(&self, entity_id: ArchRowId) -> &MaybeUninit<T> {
        let component_col = self.components.get(&T::id())
            .expect(&format!("Component {} does not exist for entity with id {}", std::any::type_name::<T>(), entity_id)); // TODO: entity id is not right
        // &component_col.components[entity_id as usize]
        let comps_ptr: *const MaybeUninit<u8> = component_col.components.as_ptr();
        let comps_ptr: *const MaybeUninit<T> = comps_ptr.cast();
        unsafe { (comps_ptr.offset(entity_id as isize)).as_ref().unwrap_unchecked() }
    }
    
    #[inline]
    pub(crate) fn get_all_components<T: Component + 'static>(&self, ent_ids: &Vec<ArchRowId>) -> Vec<&T> {
        let component_col: &ComponentColumn = self.components.get(&T::id())
            .expect(&format!("Component {} does not exist for the entities with ids {:?}", std::any::type_name::<T>(), ent_ids));
        let comps_ptr: *const MaybeUninit<u8> = component_col.components.as_ptr();
        let comps_ptr: *const MaybeUninit<T> = comps_ptr.cast();
        ent_ids.iter()
            .map(|ent_id| {
                let comp = unsafe { comps_ptr.offset(*ent_id as isize).as_ref().unwrap_unchecked() };
                unsafe { comp.assume_init_ref() }
                // let dyn_comp = &**unsafe { component_col.components[*ent_id as usize].assume_init_ref() };
                // let comp: Option<&T> = dyn_comp.as_any().downcast_ref::<T>();
                // unsafe { comp.unwrap_unchecked() }
            }).collect()
    }
    
    // #[inline]
    /// Get all components of type `T` for the entities with ids `ent_ids`
    // pub(crate) fn get_all_components_mut<T: Component + 'static>(&mut self, ent_ids: &Vec<EntityId>) -> Vec<&mut T> {
    //     let component_col: &mut ComponentColumn = self.components.get_mut(&T::id())
    //         .expect(&format!("Component {} does not exist for the entities with ids {:?}", std::any::type_name::<T>(), ent_ids));
    //     let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
    //     let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
    //     ent_ids.iter()
    //         .map(|ent_id| {
    //             let comp = unsafe { comps_ptr.offset(*ent_id as isize).as_mut().unwrap_unchecked() };
    //             unsafe { comp.assume_init_mut() }
    //             // let dyn_comp = &mut**unsafe { component_col.components[*ent_id as usize].assume_init_mut() };
    //             // let comp = dyn_comp.as_any_mut().downcast_mut::<T>();
    //             // let comp = unsafe { comp.unwrap_unchecked() };
    //             // comp as *mut T
    //         }).collect()
    // }
    
    #[inline]
    /// Get all components of type `T` for the entities with ids `ent_ids`
    pub(crate) fn get_all_components_mut_ptr<T: Component + 'static>(&mut self, ent_ids: &Vec<EntityId>) -> Vec<*mut T> {
        let component_col: &mut ComponentColumn = self.components.get_mut(&T::id())
            .expect(&format!("Component {} does not exist for the entities with ids {:?}", std::any::type_name::<T>(), ent_ids));
        let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
        let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
        ent_ids.iter()
            .map(|ent_id| {
                let comp = unsafe { comps_ptr.offset(*ent_id as isize).as_mut().unwrap_unchecked() };
                let comp: *mut T = unsafe { comp.assume_init_mut() };
                comp
                // let dyn_comp = &mut**unsafe { component_col.components[*ent_id as usize].assume_init_mut() };
                // let comp = dyn_comp.as_any_mut().downcast_mut::<T>();
                // let comp = unsafe { comp.unwrap_unchecked() };
                // comp as *mut T
            }).collect()
    }
    
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
    
    #[inline]
    pub(crate) fn has_component(&self, id: ComponentId) -> bool {
        self.components.contains_key(&id)
    }
    
    #[inline]
    pub(crate) fn remove_entity(&mut self, arch_row: ArchRowId) {
        self.available_ent_ids.push(arch_row);
    }
}
