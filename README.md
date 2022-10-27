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
kiwi-ecs = "1.2"
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

## Flags

Unit structs can't be used as Components, this is where you would have to use a flag.
Flags are represented as an enum:

```rust
#[flags]
enum Flags {
  Player,
  Enemy,
  Ground,
}
```

## Entities

To spawn a new entity with the given components:

```rust
// spawn_entity macro accepts the world as the first parameter, and the 
// components to add to the entity as the other parameters
let entity_id = spawn_entity!(world, Position { x: 0, y: 0 });
```

You can give an entity a flag using the `set_flag` method:

```rust
world.set_flag(entity_id, Flags::Player);
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
#[system(pos: Position, vel: Vel)]
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
  print_positions(&world);
  move_entities(&mut world);
  print_entity_ids(&world);
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

#[system(pos: Position)]
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
  
  let query_result = query!(world, Position);
  let query_result = query_mut!(world, Position, Velocity);
  let query_result = query!(world, EntityId, Position);
  
  // You can now loop over the components
  query_result.for_each(|components| {
    // ...
  });
}
```

<!--
Note on safety: the `query_mut` macro is unsafe, because it can cause undefined behaviour
if two of the same component types are passed in.
-->

### Flags in queries

You can further filter queries using flags:

```rust
#[system(id: EntityId, pos: Position)]
fn on_player(world: &World) {
  if world.has_flag(id, Flags::Player) {
    // ...
  }
}

let query_result = query!(world, EntityId, Position)
  .filter(|(id, _pos)| world.has_flag(*id, Flags::Player));
```

# Contributing

Contributors are always welcome. If you find any bugs, feel free to open an issue. If you feel like it, PRs are also appreciated!

# License

Licensed under the MIT license.
