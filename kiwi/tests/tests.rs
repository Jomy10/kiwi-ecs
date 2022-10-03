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
