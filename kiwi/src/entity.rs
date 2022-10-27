use crate::arch::{ArchetypeId, ArchRowId};
use crate::component::FlagId;

pub type EntityId = u32;

pub(crate) struct Entity {
    pub(crate) arch_id: ArchetypeId,
    pub(crate) arch_row: ArchRowId,
}

pub(crate) struct EntityStore {
    next_id: EntityId,
    dead: Vec<u8>,
    entities: Vec<Entity>,
    /// Flags for entities
    flags: Vec<Vec<u8>>,
}

impl EntityStore {
    pub(crate) fn new() -> Self {
        Self {
            next_id: 0,
            dead: Vec::new(),
            entities: Vec::new(),
            flags: Vec::new(),
        }
    }
    
    #[inline]
    /// Gets a new entity id
    pub(crate) fn new_id(&mut self) -> EntityId {
        let entity_id = self.next_id;
        self.next_id += 1;
        return entity_id;
    }

    #[inline]
    pub(crate) fn spawn_with_id(&mut self, ent_id: EntityId, arch_id: ArchetypeId, arch_row: ArchRowId) {
        if self.entities.len() <= ent_id as usize {
            self.entities.resize_with(ent_id as usize + 1, || Entity { arch_id, arch_row });
        } else {
            self.entities[ent_id as usize] = Entity { arch_id, arch_row };
        }
    }

    #[inline]
    /// Marks an entity as dead
    pub(crate) fn kill(&mut self, ent: EntityId) {
        let idx = ent / 8;
        let idx2 = ent % 8;
        if self.dead.len() <= ent as usize {
            self.dead.resize(ent as usize + 1, 0);
        }
        let dead_map = &mut self.dead[idx as usize];
        *dead_map |= 1 << idx2
    }

    #[inline]
    pub(crate) fn is_alive(&self, ent: EntityId) -> bool {
        let idx = ent / 8;
        let idx2 = ent % 8;
        match self.dead.get(idx as usize) {
            Some(bitmap) => {
                return bitmap & (1 << idx2) != (1 << idx2);
            }
            None => {
                return true;
            }
        }
    }

    #[inline]
    pub(crate) fn entity_count(&self) -> usize {
        (0..(self.entities.len() as u32))
            .filter(|ent_id| self.is_alive(*ent_id))
            .count()
    }
    
    #[inline]
    pub(crate) fn entities(&self) -> &Vec<Entity> {
        &self.entities
    }
    
    #[inline]
    pub(crate) fn has_flag(&self, ent: EntityId, flag: FlagId) -> bool {
        let idx = ent / 8;
        let idx2 = ent % 8;
        
        match self.flags.get(flag as usize) {
            Some(bitmaps) => {
                match bitmaps.get(idx as usize) {
                    Some(bitmap) => {
                        return bitmap & (1 << idx2) == (1 << idx2);
                    }
                    None => {
                        return false;
                    }
                }
            }
            None => {
                return false;
            }
        }
    }
    
    #[inline]
    pub(crate) fn set_flag(&mut self, ent: EntityId, flag: FlagId) {
        let idx = ent / 8;
        let idx2 = ent % 8;
        
        if self.flags.len() <= flag as usize {
            self.flags.resize_with(flag as usize + 1, Vec::new);
        }
        
        if self.flags[flag as usize].len() <= idx as usize {
            self.flags[flag as usize].resize_with(idx as usize + 1, || 0);
        }
        
        self.flags[flag as usize][idx as usize] |= 1 << idx2;
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::EntityStore;
    
    #[test]
    fn new_entity_id() {
        let mut ent_store = EntityStore::new();
        let id1 = ent_store.new_id();
        let id2 = ent_store.new_id();
        
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
    }

    #[test]
    fn kill_entity() {
        let mut ent_store = EntityStore::new();
        let id1 = ent_store.new_id();
        assert!(ent_store.is_alive(id1));
        let id2 = ent_store.new_id();
        assert!(ent_store.is_alive(id2));
        ent_store.kill(id2);
        
        assert!(ent_store.is_alive(id1));
        assert!(!ent_store.is_alive(id2));
    }
}
