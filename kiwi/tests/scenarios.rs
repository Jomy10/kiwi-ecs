use kiwi_ecs::*;

#[test]
fn spawn_fighter_game() {
    #[derive(Debug, Clone, Copy)]
    #[allow(dead_code)]
    pub struct Vec2 {
        x: f32, y: f32
    }
    
    impl Vec2 {
        pub fn new(x: f32, y: f32) -> Self {
            Self { x, y }
        }
        
        const ZERO: Self = Self { x: 0., y: 0. };
    }
    
    #[flags]
    pub enum Flags {
        Fighter,
        #[allow(unused)]
        Enemy,
    }

    #[derive(Component, Debug)]
    pub struct Position(pub Vec2);

    #[derive(Component)]
    pub struct Velocity(pub Vec2);

    #[derive(Component)]
    pub struct Speed(pub f32);

    #[derive(Component)]
    pub struct DeployPosition(pub Vec2);

    #[derive(Component)]
    /// Entity that is being targeted for fighting
    pub struct Target(pub Option<EntityId>);

    #[derive(Component)]
    pub struct Sprite(pub usize);

    #[derive(Component)]
    pub struct Bounds(pub Vec2);
    
    assert_ne!(Bounds::id(), Velocity::id());
    assert_ne!(Bounds::id(), Position::id());
    assert_ne!(Bounds::id(), Speed::id());
    assert_ne!(Bounds::id(), DeployPosition::id());
    assert_ne!(Bounds::id(), Sprite::id());
    
    assert_eq!(std::mem::size_of::<Bounds>(), std::mem::size_of::<Position>());
    
    pub fn spawn_fighter(world: &mut World, pos: Vec2) -> EntityId { 
        let id = spawn_entity!(world,
            Position(pos),
            Velocity(Vec2::ZERO),
            Bounds(Vec2::ZERO),
            Speed(30.),
            Target(None),
            DeployPosition(pos),
            Sprite(0),
        );
        
        assert_eq!(*Bounds::get_archetypes(), vec![0]);

        let id = id;
    
        world.set_flag(id, Flags::Fighter);
        
        assert_eq!(world.has_flag(id, Flags::Fighter), true);
    
        id
    }

    let mut world = World::new();
    
    assert_eq!(std::mem::size_of::<Speed>(), 4);
    
    for i in 0..1000 {
        let id = spawn_fighter(&mut world, Vec2::new(415.896, 500.6));
        assert_eq!(id, i);
    }
}
