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

macro_rules! comp {
    ($name: ident) => {
        #[derive(Component)]
        struct $name(u32);
    }
}

#[allow(unused)]
#[flags]
enum Flags {
    Flag1,
    Flag2
}

#[test]
fn spawn_multiple_entities() {
    pos_comp!();
    let mut world = World::new();
    
    comp!(C1);
    comp!(C2);
    comp!(C3);
    comp!(C4);
    comp!(C5);
    comp!(C6);
    
    for _ in 0..1000 {
        let id = spawn_entity!(world, Pos { x: 0, y: 0 }, C1(0), C2(0),C3(0),C4(0),C5(0),C6(0),);
        world.set_flag(id, Flags::Flag1);
        let id = spawn_entity!(world, Pos { x: 0, y: 0 }, C1(0), C2(0),C3(0),C4(0),C5(0),C6(0),);
        world.set_flag(id, Flags::Flag1);
    }
    
    assert_eq!(world.entity_count(), 2000);
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
    let components: Vec<&Pos> = components.collect();
    assert_eq!(components.len(), 2);

    let components = query!(world, Pos, Vel);
    let components: Vec<(&Pos, &Vel)> = components.collect();
    assert_eq!(components.len(), 1);
    
    let ids = query!(world, EntityId);
    assert_eq!(ids.len(), 3);
}

#[test]
fn get_component() {
    pos_comp!();
    
    let mut world = World::new();
    let ent_id = world.spawn_entity1(Pos{x: 1, y: 0});
    let comp: &Pos = world.get_component(ent_id);
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
    let components: Vec<&Pos> = components.collect();
    assert_eq!(components.len(), 2);
    assert!(components.contains(&&Pos { x: 0, y: 5 }));
    assert!(components.contains(&&Pos { x: 6, y: 7 }));
    
    let components = world.query2::<Vel, Pos>();
    let components: Vec<(&Vel, &Pos)> = components.collect();
    assert_eq!(components.len(), 1);
    assert!(components.contains(&(&Vel { x: 8, y: 9 }, &Pos { x: 6, y: 7 })));

    let components = world.query2::<Pos, Vel>();
    let components: Vec<(&Pos, &Vel)> = components.collect();
    assert_eq!(components.len(), 1);
    assert!(components.contains(&(&Pos { x: 6, y: 7 }, &Vel { x: 8, y: 9 })));
}

#[test]
fn mut_query() {
    pos_comp!();
    vel_comp!();
    
    let mut world = World::new();

    world.spawn_entity0();
    let _ = world.spawn_entity1(Pos { x: 0, y: 5 });
    world.spawn_entity2(Pos { x: 6, y: 7}, Vel { x: 8, y: 9 });
    
    let q  = world.query_mut2::<Vel, Pos>();
    q.for_each(|(v, _)| v.x = 1);
    
    let mut components = world.query2::<Pos, Vel>();
    assert_eq!(*components.next().unwrap().1, Vel{x: 1, y: 9})
}

#[test]
fn mut_query_after_set_component() {
    pos_comp!();
    
    let mut world = World::new();
    let world_ptr: *mut World = &mut world;
    
    spawn_entity!(world,
        Pos { x: 0, y: 0 }
    );
    
    let query = world.query_mut_ids1::<Pos>();
    query.for_each(|(id, pos)| {
        assert_eq!(*pos, Pos { x: 0, y: 0 });
        
        pos.x = 4;
        
        assert_eq!(*pos, Pos { x: 4, y: 0 });
        
        let current_pos = unsafe { (*world_ptr).get_component::<Pos>(id) };
        
        assert_eq!(*pos, *current_pos);
        
        unsafe { (*world_ptr).set_component(id, Pos { x: 6, y: 10 }) };
        
        assert_eq!(*unsafe { (*world_ptr).get_component::<Pos>(id) }, Pos { x: 6, y: 10 });
        assert_eq!(*pos, Pos { x: 6, y: 10 });
    });
}

// #[test]
// fn mut_query_ptrs_after_set_component() {
//     pos_comp!();
    
//     let mut world = World::new();
    
//     spawn_entity!(world,
//         Pos { x: 0, y: 0 }
//     );
    
//     let (ids, poss) = unsafe { query_mut!(world, EntityId, Pos) };
//     for i in 0..ids.len() {unsafe {
//         let id = ids[i];
//         let pos = poss[i];
        
//         assert_eq!(*pos, Pos { x: 0, y: 0 });
        
//         (*pos).x = 4;
        
//         assert_eq!(*pos, Pos { x: 4, y: 0 });
        
//         let current_pos = world.get_component::<Pos>(id);
        
//         assert_eq!(*pos, *current_pos);
        
//         world.set_component(id, Pos { x: 6, y: 10 });
        
//         assert_eq!(*world.get_component::<Pos>(id), Pos { x: 6, y: 10 });
//         assert_eq!(*pos, Pos { x: 6, y: 10 }); // pointer still point to the correct array index
//     }}
// }

#[test]
fn flags() {
    pos_comp!();
    
    #[flags]
    enum Flags {
        Player,
        Ground
    }
    
    let mut world = World::new();
    
    let id1 = spawn_entity!(world,
        Pos { x: 0, y: 1 }
    );
    world.set_flag(id1, Flags::Player);
    
    let id2 = spawn_entity!(world,
        Pos { x: 0, y: 1 }
    );
    world.set_flag(id2, Flags::Ground);
    
    let q = query!(world,
        EntityId,
        Pos
    ).filter(|(id, _)| {
        world.has_flag(*id, Flags::Player)
    });
    
    q.for_each(|(id, _)| assert_eq!(id, id1));
    
    let q = query!(world, EntityId, Pos);
    q.filter(|(id, _)| world.has_flag(*id, Flags::Ground))
        .for_each(|(id, _)| assert_eq!(id, id2));
}

#[test]
fn system_macro_ids() {
    pos_comp!();
    
    let mut world = World::new();
    
    spawn_entity!(world, Pos { x: 0, y: 1 });
    
    #[system(id: EntityId, pos: Pos)]
    fn test_macro(world: &World) {
        assert_eq!(id, 0);
        assert_eq!(*pos, Pos { x: 0, y: 1 });
    }
    
    test_macro(&world);
}

macro_rules! query_ids_spawn_entity {
    ($world: ident) => {
        spawn_entity!($world,
            Pos { x: 0, y: 0 },
            Vel { x: 0, y: 0 },
        )
    }
}

#[test]
fn query_ids() {
    pos_comp!();
    vel_comp!();
    
    let mut world = World::new();

    query_ids_spawn_entity!(world);
    // query_ids_spawn_entity!(world);
    spawn_entity!(world, Pos { x: 0, y: 0 });
    query_ids_spawn_entity!(world);
    query_ids_spawn_entity!(world);
    query_ids_spawn_entity!(world);
    query_ids_spawn_entity!(world);
    
    let ids: Vec<EntityId> = query!(world, EntityId, Pos, Vel)
        .map(|(id, _, _)| {
            id
        }).collect();
    assert_eq!(ids, vec![0, 2, 3, 4, 5]);
}

#[test]
fn system_single_component() {
    pos_comp!();
    
    #[system(pos: Pos)]
    fn test_pos_system(world: &mut World) {
        assert_eq!(*pos, Pos {x: 0, y: 1});
    }
    
    let mut world = World::new();
    
    spawn_entity!(world, Pos { x: 0, y: 1 });
    
    test_pos_system(&mut world);
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
}
