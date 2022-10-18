use std::collections::HashMap;

use crate::component::ComponentId;
use crate::arch::Archetype;

pub type ArchetypeId = u32;

struct CompMapVal {
    components: Vec<ComponentId>,
    archetype: ArchetypeId,
}

impl CompMapVal {
    #[inline]
    fn hash_component(comps: &mut Vec<ComponentId>) -> u32 {
        comps.sort();
        let mut total: u32 = 0;
        for comp_id in comps {
            total = total.overflowing_add(*comp_id).0;
        }
        return total;
    }
}

pub(crate) struct ArchStore {
    pub(crate) archetypes: Vec<Archetype>,
    comp_map: HashMap<u32, CompMapVal>
}

pub(crate) enum NewEntityResult {
    NewArchetype(ArchetypeId),
    OldArchetype(ArchetypeId)
}

#[cfg(test)]
impl NewEntityResult {
    fn unwrap(&self) -> ArchetypeId {
        match self {
            Self::NewArchetype(id) | Self::OldArchetype(id) => *id
        }
    }
}

impl ArchStore {
    pub(crate) fn new() -> Self {
        Self {
            archetypes: Vec::new(),
            comp_map: HashMap::new(),
        }
    }
    
    #[inline]
    pub(crate) fn get_archetype(&self, archetype: ArchetypeId) -> &Archetype {
        &self.archetypes[archetype as usize]
    }
    
    #[inline]
    pub(crate) fn get_archetype_mut(&mut self, archetype: ArchetypeId) -> &mut Archetype {
        &mut self.archetypes[archetype as usize]
    }

    #[inline]
    pub(crate) fn remove_entity(&mut self, entity: &crate::entity::Entity) {
        let arch = &mut self.archetypes[entity.arch_id as usize];
        arch.remove_entity(entity.arch_row);
    }
    
    #[inline]
    /// Get the archetype of a new entity
    /// - `get_component_sizes` is called when a new archetype is created
    pub(crate) fn get_new_entity_archetype(&mut self, components: Vec<ComponentId>, get_component_sizes: fn() -> Vec<usize>) -> NewEntityResult {
        let mut components = components;
        let comps_hash = CompMapVal::hash_component(&mut components); 
        
        return self.get_archetype_id_for_component_hash(comps_hash, &components, get_component_sizes);
    }

    #[inline]
    fn get_archetype_id_for_component_hash(&mut self, hash: u32, components: &Vec<ComponentId>, get_component_sizes: fn() -> Vec<usize>) -> NewEntityResult {
        match self.comp_map.get(&hash) {
            Some(val) => {
                // See if correct
                if &val.components != components {
                    return self.get_archetype_id_for_component_hash(hash + 1, components, get_component_sizes);
                } else {
                    return NewEntityResult::OldArchetype(val.archetype);
                }
            }
            None => {
                // Create new archetype
                let id = self.archetypes.len() as ArchetypeId;
                self.archetypes.push(
                    Archetype::new(components, &get_component_sizes())
                );
                self.comp_map.insert(hash, CompMapVal {
                    components: components.clone(), // doesn't happen often
                                // No need to shrink components vec; if using `vec!`, it is already the correct size and `vec.push` doesn't allocate a lot at a time
                    archetype: id,
                });
                return NewEntityResult::NewArchetype(id);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::ArchStore;
    
    #[test]
    fn get_archetype_id_for_components_list() {
        let mut arch_store = ArchStore::new();
        let id1 = arch_store.get_new_entity_archetype(vec![0, 1], || { vec![1, 1]}).unwrap();
        let id2 = arch_store.get_new_entity_archetype(vec![0, 2], || { vec![1, 1]}).unwrap();
        let id3 = arch_store.get_new_entity_archetype(vec![1, 0], || { vec![1, 1]}).unwrap();
        let id4 = arch_store.get_new_entity_archetype(vec![], || { vec![] }).unwrap();
        let id5 = arch_store.get_new_entity_archetype(vec![], || { vec![] }).unwrap();
        
        assert_eq!(id1, id3);
        assert_ne!(id2, id1);
        assert_eq!(id3, id1);
        assert_eq!(id4, id5);
        assert_ne!(id4, id3);
        assert_ne!(id4, id2);
    }
}
