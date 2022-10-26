use std::time::SystemTime;
use kiwi_ecs::*;

#[derive(Component)]
#[allow(unused)]
struct Pos {
    x: u32, y: u32
}

#[derive(Component)]
#[allow(unused)]
struct Vel {
    x: u32, y: u32
}

const ENT_SIZE: usize = 100_000;
const ITER_COUNT: usize = 1000;

fn main() {
    let mut world = World::new();
    
    for _ in 0..ENT_SIZE {
        spawn_entity!(
            world,
            Pos { x: 0, y: 0 },
            Vel { x: 1, y: 1 },
        );
    }
    
    for _ in 0..ITER_COUNT {
        let start = SystemTime::now();
        
        let query = world.query2::<Pos, Vel>();
        query.for_each(|(pos, vel)| {
            let _ = pos;
            let _ = vel;
        });
        
        let dt = SystemTime::now().duration_since(start).unwrap();
        println!("{:?}", dt);
    }
}
