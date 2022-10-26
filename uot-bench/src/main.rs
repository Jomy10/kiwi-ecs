use std::time::SystemTime;

use kiwi_ecs::*;
use rand::prelude::*;
use std::collections::HashSet;

#[derive(Component)]
struct Vel {
    x: f64, y: f64
}

#[derive(Component)]
struct Pos {
    x: f64, y: f64
}

#[derive(Component)]
#[derive(Debug)]
struct Collider {
    radius: f64,
    count: u32,
}

fn main() {
    bench(10000, 0);
}

fn bench(size: usize, collision_limit: u32) {
    let iterations = 1000;

    let mut world = World::new();

    let max_speed = 10.0;
    let max_position = 100.0;
    let max_collider = 1.0;
    
    let mut rng = rand::thread_rng();
    
    for _ in 0..size {
        world.spawn_entity3(
            Pos {
                x: max_position * rng.gen::<f64>(),
                y: max_position * rng.gen::<f64>()
            },
            Vel {
                x: max_speed * rng.gen::<f64>(),
                y: max_speed * rng.gen::<f64>()
            },
            Collider {
                radius: max_collider * rng.gen::<f64>(),
                count: 0
            }
        );
    }
    
    let fixed_time = 0.015;
    
    #[allow(unused_assignments)] // Don't know why a warning occurs here
    let mut start = SystemTime::now();
    
    // let dt = SystemTime::now().duration_since(start);
    
    for iter_count in 0..iterations {
        start = SystemTime::now();
        
        move_circles(&mut world, fixed_time, max_position);
        
        let mut death_count = 0;
        
        unsafe { check_collisions(&mut world, collision_limit, &mut death_count); }
        
        // Spawn new entities, one per each entiy we deleted
        // println!("Spawning {death_count} entities");
        for _ in 0..death_count {
            world.spawn_entity3(
                Pos {
                    x: max_position * rng.gen::<f64>(),
                    y: max_position * rng.gen::<f64>()
                },
                Vel {
                    x: max_position * rng.gen::<f64>(),
                    y: max_position * rng.gen::<f64>()
                },
                Collider {
                    radius: max_collider * rng.gen::<f64>(),
                    count: 0
                }
            );
        }
        
        let dt = SystemTime::now().duration_since(start).unwrap();
        // println!("{} {:?} {}", iter_count, dt, death_count);
        println!("{} {:?}", iter_count, dt);
        // dbg!(world.entity_count());
    }
}

#[system(pos: Pos, vel: Vel)]
fn move_circles(world: &mut World, fixed_time: f64, max_position: f64) {
    pos.x += vel.x * fixed_time;
    pos.y += vel.y * fixed_time;
    
    // Bump into the bounding rect
    if pos.x <= 0.0 || pos.x >= max_position {
        vel.x = -vel.x;
    }
    if pos.y <= 0.0 || pos.y >= max_position {
        vel.y = -vel.y;
    }
}

unsafe fn check_collisions(world: &mut World, collision_limit: u32, death_count: &mut u32) {
    // let (ids, pos, col): (Vec<EntityId>, Vec<*mut Pos>, Vec<*mut Collider>) = world.query_mut_ptr_ids2::<Pos, Collider>();
    let mut dead = HashSet::<EntityId>::new();
    let world_ptr: *const World = &*world;
    // for i in 0..pos.len() {
    // query.clone()
    //     .zip(query)
    //     .filter(|((id1, _, _), (id2, _, _))| {
    //         id1 == id2
    //     })
    //     .for_each(|((id1, pos1, col1), (id2, pos2, col2))| {
    //     });
    
    let mut query: Vec<(EntityId, &mut Pos, &mut Collider)> = query_mut!(world, EntityId, Pos, Collider).collect();
    let query_ptr: *mut Vec<_> = &mut query;
    
    (*query_ptr).iter_mut().for_each(|(id1, pos1, col1)| {
        query.iter().for_each(|(id2, pos2, col2)| {
            if id1 != id2 && !dead.contains(&id1) && !dead.contains(&id2) {
                let dx = pos1.x - pos2.x;
                let dy = pos2.y - pos2.y;
                let dist_sq = (dx * dx) + (dy * dy);
                
                let dr = col1.radius - col2.radius;
                let dr_sq = dr * dr;
                
                if dr_sq > dist_sq {
                    (*col1).count += 1;
                }
                
                // kill and spawn one
                if collision_limit > 0 && col1.count > collision_limit {
                    *death_count += 1;
                    dead.insert(*id1);
                }
            }
        });
    });
    
    // query.for_each(|(id, pos1, col1)| {
    //         if id == j { continue }
    //         if dead.contains(&i) || dead.contains(&j) {
    //             continue;
    //         }
            
    //         let (pos2, col2): (*const Pos, *const Collider) = (pos[j], col[j]);
            
    //         let dx = (*pos1).x - (*pos2).x;
    //         let dy = (*pos2).y - (*pos2).y;
    //         let dist_sq = (dx * dx) + (dy * dy);
            
    //         let dr = (*col1).radius - (*col2).radius;
    //         let dr_sq = dr *dr;
            
    //         if dr_sq > dist_sq {
    //             (*col1).count += 1;
    //         }
            
    //         // Kill and spawn one
    //         if collision_limit > 0 && (*col1).count > collision_limit {
    //             *death_count += 1;
    //             dead.insert(i);
    //         }
    //     }
    // }); // outer loop
    
    // Kill entities
    dead.iter().for_each(|dead_id| {
        world.kill(*dead_id);
    })
}

// #[system(collider: Collider)]
// fn print_collider_count(world: &World) {
//     println!("{} {}", __kiwi_ent_id, collider.count);
// }
