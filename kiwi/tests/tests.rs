use kiwi_ecs::*;

macro_rules! pos_comp {
    () => {
        #[derive(Debug, PartialEq, Component)]
        struct Pos {
            x: u32, y: u32
        }
    }
}

macro_rules! vel_comp {
    () => {
        #[derive(Debug, PartialEq, Component)]
        struct Vel {
            x: u32, y: u32
        }
    }
}
/*
#[test]
fn kill_entity_query() {
    let mut world = World::new();
    let id1 = world.spawn_entity0();
    let id2 = world.spawn_entity0();
    let ids = world.query_ids0();
    assert_eq!(ids, vec![id1, id2]);
    
    world.kill(id1);
    let ids = world.query_ids0();
    assert_eq!(ids, vec![id2]);
    
    world.kill(id2);
    let ids = world.query_ids0();
    assert_eq!(ids.len(), 0);
}

#[test]
fn spawn_entity() {
    pos_comp!();

    let mut world = World::new();
    world.spawn_entity0();
    world.spawn_entity1(Pos{x: 0, y: 0});
}

#[test]
fn spawn_entity_macro() {
    pos_comp!();
    vel_comp!();
    
    let mut world = World::new();
    spawn_entity!(world);
    spawn_entity!(world, Pos { x: 1, y: 2 });
    spawn_entity!(world, Pos { x: 3, y: 4 }, Vel { x: 5, y: 6 });
    
    let components = query!(world, Pos);
    assert_eq!(components.len(), 2);

    let components = query!(world, Pos, Vel);
    assert_eq!(components.0.len(), 1);
    assert_eq!(components.1.len(), 1);
    
    let ids = query!(world, EntityId);
    assert_eq!(ids.len(), 3);
}

#[test]
fn get_component() {
    pos_comp!();
    
    let mut world = World::new();
    let ent_id = world.spawn_entity1(Pos{x: 1, y: 0});
    let comp: &Pos = unsafe { world.get_component(ent_id) };
    assert_eq!(*comp, Pos{x: 1, y: 0});
}


#[test]
fn query() {
    pos_comp!();
    vel_comp!();
    
    let mut world = World::new();
    
    world.spawn_entity0();
    let _ = world.spawn_entity1(Pos { x: 0, y: 5 });
    world.spawn_entity2(Pos { x: 6, y: 7}, Vel { x: 8, y: 9 });
    
    let components = world.query1::<Pos>();
    assert_eq!(components.len(), 2);
    assert!(components.contains(&&Pos { x: 0, y: 5 }));
    assert!(components.contains(&&Pos { x: 6, y: 7 }));
    
    let components = world.query2::<Vel, Pos>();
    assert_eq!(components.0.len(), 1);
    assert_eq!(components.1.len(), 1);
    assert!(components.0.contains(&&Vel { x: 8, y: 9 }));
    assert!(components.1.contains(&&Pos { x: 6, y: 7 }));

    let components = world.query2::<Pos, Vel>();
    assert_eq!(components.0.len(), 1);
    assert_eq!(components.1.len(), 1);
    assert!(components.0.contains(&&Pos { x: 6, y: 7 }));
    assert!(components.1.contains(&&Vel { x: 8, y: 9 }));
}

#[test]
fn mut_query() {
    pos_comp!();
    vel_comp!();
    
    let mut world = World::new();

    world.spawn_entity0();
    let _ = world.spawn_entity1(Pos { x: 0, y: 5 });
    world.spawn_entity2(Pos { x: 6, y: 7}, Vel { x: 8, y: 9 });
    
    unsafe {
        let (vel, _): (Vec<*mut Vel>, Vec<*mut Pos>) = world.query_mut_ptr2::<Vel, Pos>();
        vel.into_iter().for_each(|v| (*v).x = 1);
    }
    
    let components = world.query2::<Pos, Vel>();
    assert!(components.1.contains(&&Vel{x: 1, y: 9}))
}

#[test]
fn mut_query_ptrs_after_set_component() {
    pos_comp!();
    
    let mut world = World::new();
    
    spawn_entity!(world,
        Pos { x: 0, y: 0 }
    );
    
    let (ids, poss) = unsafe { query_mut!(world, EntityId, Pos) };
    for i in 0..ids.len() {unsafe {
        let id = ids[i];
        let pos = poss[i];
        
        assert_eq!(*pos, Pos { x: 0, y: 0 });
        
        (*pos).x = 4;
        
        assert_eq!(*pos, Pos { x: 4, y: 0 });
        
        let current_pos = world.get_component::<Pos>(id);
        
        assert_eq!(*pos, *current_pos);
        
        world.set_component(id, Pos { x: 6, y: 10 });
        
        assert_eq!(*world.get_component::<Pos>(id), Pos { x: 6, y: 10 });
        assert_eq!(*pos, Pos { x: 6, y: 10 }); // pointer still point to the correct array index
    }}
}
*/
#[test]
fn new_query() {
    pos_comp!();
    vel_comp!();
    
    let mut world = World::new();
    
    spawn_entity!(world,
        Pos { x: 0, y: 1 },
        Vel { x: 2, y: 3}
    );
    
    spawn_entity!(world,
        Pos { x: 4, y: 5 }
    );
    
    let q = world.temp_query::<Pos, Vel>(); // temp_query is a temporary name
    q.for_each(|v| {
         assert_eq!(*v.0, Pos{ x: 0, y: 1 });
         assert_eq!(*v.1, Vel{ x: 2, y: 3 });
    });
}
/*
#[test]
// Flags test
fn unit_struct() {
    #[derive(Component)]
    struct Flag;
    
    let mut world = World::new();
    
    spawn_entity!(world,
        Flag,
    );
    
    let _ = query!(world, Flag);
}

mod example {
    use super::*;
    
    #[derive(Component, PartialEq, Debug)]
    struct Pos {
        x: u32, y: u32
    }
    #[derive(Component)]
    struct Vel {
        x: u32, y: u32
    }

    #[test]
    fn example() {
        let mut world = World::new();
        
        for _ in 0..10 {
            spawn_entity!(world, Pos { x: 0, y: 0 }, Vel { x: 1, y: 1 });
        }
        
        move_entities(&mut world);
        
        let components = query!(world, Pos);
        for component in components {
            assert_eq!(*component, Pos { x: 1, y: 1 });
        }
    }
    
    #[system(pos: Pos, vel: Vel)]
    fn move_entities(world: &mut World) {
        pos.x += vel.x;
        pos.y += vel.y;
    }
}*/
