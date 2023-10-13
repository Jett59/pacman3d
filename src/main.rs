use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use maze::Maze;
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

#[derive(Component)]
struct Player;

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

    let mut object = GameObject::default();
    object.add_mesh(object::Mesh {
        shape: object::Shape::Box {
            width: 0.5,
            height: 1.0,
            depth: 0.5,
        },
        color: Color::BLUE,
        position: Default::default(),
        rotation: Default::default(),
    });
    object
        .spawn(
            Transform::from_xyz(0.0, 1.0, 0.0),
            RigidBody::Dynamic,
            &mut commands,
            &mut meshes,
            &mut materials,
        )
        .insert(Player)
        .insert(LockedAxes::ROTATION_LOCKED)
        .add_child(camera);

    let maze = Maze::new(&[((-10.0, 0.0), (10.0, 0.0)), ((0.0, -10.0), (0.0, 10.0))]);
    maze.create_game_object().spawn(
        Default::default(),
        RigidBody::Fixed,
        &mut commands,
        &mut meshes,
        &mut materials,
    );
}

fn player_movement(
    mut player: Query<(&mut Velocity, &mut Transform), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    const SPEED: f32 = 3.0;
    let x_velocity = if keyboard_input.pressed(KeyCode::Left) {
        -SPEED
    } else if keyboard_input.pressed(KeyCode::Right) {
        SPEED
    } else {
        0.0
    };
    let z_velocity = if keyboard_input.pressed(KeyCode::Up) {
        -SPEED
    } else if keyboard_input.pressed(KeyCode::Down) {
        SPEED
    } else {
        0.0
    };
    for (mut velocity, _) in player.iter_mut() {
        velocity.linvel.x = x_velocity;
        velocity.linvel.z = z_velocity;
    }

    let rotation = if keyboard_input.pressed(KeyCode::Left) {
        Some(PI / 2.0)
    } else if keyboard_input.pressed(KeyCode::Right) {
        Some(-PI / 2.0)
    } else if keyboard_input.pressed(KeyCode::Back) {
        Some(PI)
    } else if keyboard_input.pressed(KeyCode::Up) {
        Some(0.0)
    } else {
        None
    };
    if let Some(rotation) = rotation {
        for (_, mut transform) in player.iter_mut() {
            transform.rotation = Quat::from_rotation_y(rotation);
        }
    }
}
