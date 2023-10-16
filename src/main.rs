use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use maze::{Maze, HALF_PATH_WIDTH};
use object::GameObject;

mod maze;
mod object;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Update, player_movement)
        .run();
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Direction {
    Left,
    Right,
    #[default]
    Forward,
    Backward,
}

impl Direction {
    fn x_velocity(&self) -> f32 {
        match self {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            _ => 0.0,
        }
    }

    fn z_velocity(&self) -> f32 {
        match self {
            Direction::Forward => -1.0,
            Direction::Backward => 1.0,
            _ => 0.0,
        }
    }

    fn get_rotation(&self) -> Quat {
        match self {
            Direction::Forward => Quat::from_rotation_y(0.0),
            Direction::Left => Quat::from_rotation_y(PI / 2.0),
            Direction::Backward => Quat::from_rotation_y(PI),
            Direction::Right => Quat::from_rotation_y(PI * 1.5),
        }
    }

    fn rotate_left(&self) -> Self {
        match self {
            Direction::Forward => Direction::Left,
            Direction::Left => Direction::Backward,
            Direction::Backward => Direction::Right,
            Direction::Right => Direction::Forward,
        }
    }

    fn rotate_right(&self) -> Self {
        match self {
            Direction::Forward => Direction::Right,
            Direction::Right => Direction::Backward,
            Direction::Backward => Direction::Left,
            Direction::Left => Direction::Forward,
        }
    }

    fn rotate_backward(&self) -> Self {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Clone, Debug, Default, Component)]
struct Player {
    current_direction: Direction,
    queued_direction: Option<Direction>,
}

fn setup_graphics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
    });
    let camera = commands
        .spawn(Camera3dBundle {
            transform: Transform::default().looking_to(-Vec3::Z, Vec3::Y),
            ..Default::default()
        })
        .id();

    let mut ground = GameObject::default();
    ground.add_mesh(object::Mesh {
        color: Color::YELLOW,
        shape: object::Shape::Box {
            width: 100.0,
            height: 1.0,
            depth: 100.0,
        },
        position: Default::default(),
        rotation: Default::default(),
    });
    ground.spawn(
        Transform::default(),
        RigidBody::Fixed,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    const PLAYER_RADIUS: f32 = 0.25;

    let mut player = GameObject::default();
    player.add_mesh(object::Mesh {
        shape: object::Shape::Cylinder {
            radius: PLAYER_RADIUS,
            height: 1.0,
        },
        color: Color::BLUE,
        position: Default::default(),
        rotation: Default::default(),
    });
    player
        .spawn(
            Transform::from_xyz(0.0, 1.0, 0.0),
            RigidBody::Dynamic,
            &mut commands,
            &mut meshes,
            &mut materials,
        )
        .insert(Player::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        .add_child(camera);

    let maze = Maze::new(&[
        ((-10.0, 0.0), (10.0, 0.0)),
        ((0.0, -10.0), (0.0, 10.0)),
        ((-10.0, 10.0), (10.0, 10.0)),
        ((-10.0, -10.0), (10.0, -10.0)),
        ((10.0, -10.0), (10.0, 10.0)),
        ((-10.0, -10.0), (-10.0, 10.0)),
        ((-5.0, 10.0), (-5.0, 20.0)),
        ((-5.0, 20.0), (-15.0, 20.0)),
        ((-15.0, 20.0), (-15.0, 5.0)),
        ((-15.0, 5.0), (-10.0, 5.0)),
        ((5.0, 10.0), (5.0, 20.0)),
        ((5.0, 20.0), (15.0, 20.0)),
        ((15.0, 20.0), (15.0, 5.0)),
        ((15.0, 5.0), (10.0, 5.0)),
        ((5.0, -10.0), (5.0, -20.0)),
        ((5.0, -20.0), (15.0, -20.0)),
        ((15.0, -20.0), (15.0, -5.0)),
        ((15.0, -5.0), (10.0, -5.0)),
        ((-5.0, -10.0), (-5.0, -20.0)),
        ((-5.0, -20.0), (-15.0, -20.0)),
        ((-15.0, -20.0), (-15.0, -5.0)),
        ((-15.0, -5.0), (-10.0, -5.0)),
        ((-15.0, 5.0), (-15.0, -5.0)),
        ((15.0, 5.0), (15.0, -5.0)),
    ]);
    println!("{:#?}", maze);
    maze.create_game_object().spawn(
        Default::default(),
        RigidBody::Fixed,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    // We need to detect when the player is intersecting with an intersection, since they can only move when this is the case.
    for intersection in maze.intersections() {
        commands
            .spawn(Collider::ball(HALF_PATH_WIDTH - PLAYER_RADIUS * 2.0))
            .insert(Sensor)
            .insert(Transform::from_xyz(
                intersection.coordinates.0,
                0.0,
                intersection.coordinates.1,
            ))
            .insert(GlobalTransform::default());
    }
}

fn player_movement(
    mut player: Query<(&mut Player, &mut Velocity, &mut Transform, Entity)>,
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
) {
    for (mut player, mut velocity, mut transform, entity) in player.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Down) {
            player.current_direction = player.current_direction.rotate_backward();
        }
        let is_at_intersection = rapier_context.intersections_with(entity).next().is_some();
        if keyboard_input.just_pressed(KeyCode::Left) {
            if is_at_intersection {
                player.current_direction = player.current_direction.rotate_left();
            } else {
                player.queued_direction = Some(player.current_direction.rotate_left());
            }
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            if is_at_intersection {
                player.current_direction = player.current_direction.rotate_right();
            } else {
                player.queued_direction = Some(player.current_direction.rotate_right());
            }
        }
        if is_at_intersection {
            if let Some(queued_direction) = player.queued_direction {
                player.current_direction = queued_direction;
                player.queued_direction = None;
            }
        }
        const SPEED: f32 = 3.0;
        velocity.linvel.x = player.current_direction.x_velocity() * SPEED;
        velocity.linvel.z = player.current_direction.z_velocity() * SPEED;
        transform.rotation = player.current_direction.get_rotation();
    }
}
