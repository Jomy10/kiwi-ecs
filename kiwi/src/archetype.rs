use std::mem::MaybeUninit;
use std::collections::HashMap;

use crate::component::{Component, ComponentId};
use crate::entity::{EntityId, EntityStore};

//=====================
// ID Type
//=====================

pub(crate) type ArchRowId = u32;

//=====================
// Component storage
//=====================

struct ComponentColumn {
    components: Vec<MaybeUninit<u8>>,
}

impl ComponentColumn {
    fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }
}

struct ComponentColumnWrapper {
    val: Option<ComponentColumn>,
    /// The size of the component in bytes
    size: usize
}

impl ComponentColumnWrapper {
    fn new(size: usize) -> Self {
        Self {
            val: if size == 0 {
                None
            } else {
                Some(ComponentColumn::new())
            },
            size,
        }
    }
}

//=====================
// Archetype
//=====================

pub(crate) struct Archetype {
    components: HashMap<ComponentId, ComponentColumnWrapper>,
    available_ent_ids: Vec<ArchRowId>,
    entities: Vec<EntityId>,
}

impl Archetype {
    pub(crate) fn new(components: &Vec<ComponentId>, sizes: &Vec<usize>) -> Self {
        // let comps: Vec<ComponentColumn> = Vec::with_capacity(components.len());
        let mut comps = HashMap::with_capacity(components.len());
        // for component in components {
        for i in 0..components.len() {
            comps.insert(components[i], ComponentColumnWrapper::new(sizes[i]));
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
        let component_col_wrap = self.components.get_mut(&T::id())
            .expect(&format!("Component {} does not exist for entity with id {}", std::any::type_name::<T>(), entity_id));

        if component_col_wrap.size == 0 {
            return;
        }
        // size is not 0, so is component column type
        let component_col = unsafe { component_col_wrap.val.as_mut().unwrap_unchecked() };
        
        // should make the function safe + checks above (size != 0 && component exists for this archetype)
        if component_col.components.len() <= entity_id as usize * component_col_wrap.size {
            component_col.components.resize_with((entity_id as usize) * component_col_wrap.size + component_col_wrap.size, MaybeUninit::uninit);
        }
        let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
        let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
        unsafe { *(comps_ptr.offset(entity_id as isize)) = MaybeUninit::new(component); }
    }
    
    #[inline]
    /// Get component of type `T` for entity with arch row `entity_id`
    ///
    /// # Safety
    /// No checks are performed whether the component is a unit struct
    pub(crate) unsafe fn get_component<T: Component + 'static>(&self, entity_id: ArchRowId) -> &T {
        let component_col_wrap = self.components.get(&T::id())
            .expect(&format!("Component {} does not exist for entity with id {}", std::any::type_name::<T>(), entity_id)); // TODO: entity id is not right

        // &component_col.components[entity_id as usize]
        let component_col = component_col_wrap.val.as_ref().unwrap_unchecked();
        let comps_ptr: *const MaybeUninit<u8> = component_col.components.as_ptr();
        let comps_ptr: *const MaybeUninit<T> = comps_ptr.cast();
        (comps_ptr.offset(entity_id as isize)).as_ref().unwrap_unchecked().assume_init_ref()
    }
    
    #[inline]
    /// # Safety
    /// No checks are performed whether the component is a flag
    pub(crate) unsafe fn get_component_mut<T: Component + 'static>(&mut self, entity_id: ArchRowId) -> &mut T {
        let component_col_wrap = self.components.get_mut(&T::id())
            .expect(&format!("Component {} does not exist for entity with id {}", std::any::type_name::<T>(), entity_id)); // TODO: entity id is not right
        
        let component_col = component_col_wrap.val.as_mut().unwrap_unchecked();
        let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
        let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
        (comps_ptr.offset(entity_id as isize)).as_mut().unwrap_unchecked().assume_init_mut()
    }
    
    #[inline]
    pub(crate) unsafe fn get_all_components
        <'a, T: Component + 'static>
        (
            &'a self, 
            ent_ids: impl std::iter::Iterator<Item = ArchRowId>
        )
        -> impl std::iter::Iterator<Item = &'a T>
    {
        let component_col_wrap = self.components.get(&T::id())
            .expect(&format!("Component {} does not exist for the given entities", std::any::type_name::<T>()));
            // .expect(&format!("Component {} does not exist for the entities with ids {:?}", std::any::type_name::<T>(), ent_ids));
        
        let component_col = component_col_wrap.val.as_ref().unwrap_unchecked();
        let comps_ptr: *const MaybeUninit<u8> = component_col.components.as_ptr();
        let comps_ptr: *const MaybeUninit<T> = comps_ptr.cast();
        
        ent_ids.into_iter()
            .map(move |ent_id| { // move comps_ptr
                let comp = comps_ptr.offset(ent_id as isize).as_ref().unwrap_unchecked();
                comp.assume_init_ref()
            })
    }

    #[inline]
    pub(crate) unsafe fn get_all_components_mut<
        'a, T: Component + 'static
    >(
        &'a mut self,
        ent_ids: impl std::iter::Iterator<Item = ArchRowId>,
    ) -> impl std::iter::Iterator<Item = &'a mut T> 
    {
        let component_col_wrap = self.components.get_mut(&T::id())
            .expect(&format!("Component {} does not exist for the given entities", std::any::type_name::<T>()));
        
        let component_col = component_col_wrap.val.as_mut().unwrap_unchecked();
        let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
        let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
        
        ent_ids
            .map(move |ent_id| {
                let comp = comps_ptr.offset(ent_id as isize).as_mut().unwrap_unchecked();
                comp.assume_init_mut()
            })
    }
    
    #[inline]
    #[allow(unused)] // TODO: use again in query_mut_ptr
    /// Get all components of type `T` for the entities with ids `ent_ids`
    pub(crate) unsafe fn get_all_components_mut_ptr<T: Component + 'static>(&mut self, ent_ids: &Vec<EntityId>) -> Vec<*mut T> {
        let component_col_wrap = self.components.get_mut(&T::id())
            .expect(&format!("Component {} does not exist for the entities with ids {:?}", std::any::type_name::<T>(), ent_ids));
        
        let component_col = component_col_wrap.val.as_mut().unwrap_unchecked();
        let comps_ptr: *mut MaybeUninit<u8> = component_col.components.as_mut_ptr();
        let comps_ptr: *mut MaybeUninit<T> = comps_ptr.cast();
        ent_ids.iter()
            .map(|ent_id| {
                let comp = comps_ptr.offset(*ent_id as isize).as_mut().unwrap_unchecked();
                let comp: *mut T = comp.assume_init_mut();
                comp
                // let dyn_comp = &mut**unsafe { component_col.components[*ent_id as usize].assume_init_mut() };
                // let comp = dyn_comp.as_any_mut().downcast_mut::<T>();
                // let comp = unsafe { comp.unwrap_unchecked() };
                // comp as *mut T
            }).collect()
    }
    
    #[inline]
    pub(crate) fn get_arch_rows<'a, 'b>(&'a self, ent_store: &'b EntityStore) 
        -> impl std::iter::Iterator<Item = ArchRowId> + 'a
        where 'b: 'a
    {
        self.entities.iter()
            .enumerate()
            .filter(|(_, id)| ent_store.is_alive(**id))
            .map(|(row, _)| row as u32)
    }
    
    #[inline]
    pub(crate) fn get_entity_ids<'a, 'b>(&'a self, ent_store: &'b EntityStore)
        -> impl std::iter::Iterator<Item = EntityId> + 'a
        where 'b: 'a
    {
        self.entities.iter()
            .filter(|id| ent_store.is_alive(**id))
            .map(|id| *id)
    }
    
    // #[inline]
    // pub(crate) fn get_rows_and_ids(&self, ent_store: &EntityStore) -> Vec<(ArchRowId, EntityId)> {
    //     get_entity_ids_enumerate_iter!(self, ent_store)
    //         .map(|(row, id)| (row as u32, *id))
    //         .collect()
    // }
    
    #[inline]
    pub(crate) fn has_component(&self, id: ComponentId) -> bool {
        self.components.contains_key(&id)
    }
    
    #[inline]
    pub(crate) fn remove_entity(&mut self, arch_row: ArchRowId) {
        self.available_ent_ids.push(arch_row);
    }
}
