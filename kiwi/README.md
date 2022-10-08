<p align="center">
  <img src="https://raw.githubusercontent.com/Jomy10/kiwi-ecs/master/logo.png" alt="kiwi ecs">
</p>

<p align="center">
  A performant, zero-dependency ECS library with a nice API written in Rust.
</p>

# Usage
```toml
# Cargo.toml

[dependecies]
kiwi-ecs = "1.0"
```

```rust
// lib.rs
use kiwi_ecs::*;
```

## The world

To start, create a new `World`. This is the starting point of the ecs.
The program can have multiple independent worlds.

```rust
pub fn main() {
  let mut world = World::new();
}
```

## Components

Components are defined as follows:

```rust
#[derive(Component)]
struct Position {
  x: u32,
  y: u32
}
```

## Entities

To spawn a new entity with the given ids:

```rust
// spawn_entity macro accepts the world as the first parameter, and the 
// components to add to the entity as the other parameters
let id = spawn_entity!(world, Pos { x: 0, y: 0 });
```

## Systems

There are two ways to define systems.

### The first is using the `system` macro:

```rust
// immutable system
#[system(pos: Position)]
fn print_positions(world: &World) {
  println!("{:?}", pos);
}

// mutable system
#[system(pos: Pos, vel: Vel)]
fn move_entities(world: &mut World) {
  pos.x += vel.x;
  pos.y += vel.y
}

// query entity ids as well
#[system(id: EntityId, pos: Position)]
/// prints all entities ids having the position component
fn print_entity_ids(world: &World) {
  println!("{id}");
}

pub fn main() {
  let mut world = World::new();
  
  //--snip
  
  // Call the systems
  move_entities(&mut world);
  print_positions(&world);
}
```

To create a mutable system, the function should contain `world: &mut World` as its first argument,
for an immutable one, add `world: &World`.

The function can contain any number of arguments you can pass to it when calling.

The function can return any type of `Result<(), Any>`. If this function has the given result
return type, `Ok(())` will be returned at the end of the system.

<!-- TODO: better example
**Example**:
```rust
use ggez::{graphics, Context};
use glam::Vec2;

#[system(pos: Pos)]
fn draw_pos(world: &World, canvas: &mut graphics::Canvas, ctx: &mut Context) -> GameResult<()> {
  let rectangle = graphics::Mesh::new_rectangle(
    ctx,
    graphics::DrawMode::fill(),
    graphics:Rect {
      x: 0.0,
      y: 0.0,
      w: 10.0,
      h: 10.0
    },
    graphics::Color::BLUE
  )?; // return an error if one occurs
  
  canvas.draw(&rectangle, Vec2::new(pos.x. pos.y));
} // Ok(()) is automatically returned after all entities have been queried
```
-->

### The second is using the `query` and `query_mut` macros:

```rust
pub fn main() {
  let mut world = World::new();
  
  //--snip
  
  let components: Vec<&Position> = query!(world, Position);
  let components: (Vec<*mut Position>, Vec<*mut Vel>) = unsafe { query_mut!(world, Position, Vel) };
  let components: (Vec<EntityId>, Vec<&Position>) = query!(world, EntityId, Position);
  
  // You can now loop over the components
}
```

Note on safety: the `query_mut` macro is unsafe, because it can cause undefined behaviour
if two of the same component types are passed in.

# License

Licensed under the MIT license.
