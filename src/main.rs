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

#[derive(Clone, Debug, Default, Component)]
struct Player {
    // I couldn't find a way to get rapier to stop decreasing the velocity, so we need to keep track of what it should be and reset it every frame.
    x_velocity: f32,
    z_velocity: f32,
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
    ]);
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
        if let Some(_intersection) = rapier_context.intersections_with(entity).next() {
            const SPEED: f32 = 3.0;
            let mut x_velocity = if keyboard_input.just_pressed(KeyCode::Left) {
                Some(-SPEED)
            } else if keyboard_input.just_pressed(KeyCode::Right) {
                Some(SPEED)
            } else {
                None
            };
            let mut z_velocity = if keyboard_input.just_pressed(KeyCode::Up) {
                Some(-SPEED)
            } else if keyboard_input.just_pressed(KeyCode::Down) {
                Some(SPEED)
            } else {
                None
            };
            if x_velocity.is_some() {
                z_velocity = Some(0.0);
            } else if z_velocity.is_some() {
                x_velocity = Some(0.0);
            }
            let x_velocity = x_velocity.unwrap_or(player.x_velocity);
            let z_velocity = z_velocity.unwrap_or(player.z_velocity);
            player.x_velocity = x_velocity;
            player.z_velocity = z_velocity;
            velocity.linvel.x = x_velocity;
            velocity.linvel.z = z_velocity;

            let rotation = if keyboard_input.just_pressed(KeyCode::Left) {
                Some(PI / 2.0)
            } else if keyboard_input.just_pressed(KeyCode::Right) {
                Some(-PI / 2.0)
            } else if keyboard_input.just_pressed(KeyCode::Down) {
                Some(PI)
            } else if keyboard_input.just_pressed(KeyCode::Up) {
                Some(0.0)
            } else {
                None
            };
            if let Some(rotation) = rotation {
                transform.rotation = Quat::from_rotation_y(rotation);
            }
        } else {
            velocity.linvel.x = player.x_velocity;
            velocity.linvel.z = player.z_velocity;
        }
    }
}
