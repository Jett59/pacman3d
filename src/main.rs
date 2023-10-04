use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use object::GameObject;

mod object;
mod maze;

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
        Transform::from_xyz(0.0, -2.0, 0.0),
        RigidBody::Fixed,
        &mut commands,
        &mut meshes,
        &mut materials,
    );

    let mut object = GameObject::default();
    object.add_mesh(object::Mesh {
        shape: object::Shape::Sphere { radius: 0.5 },
        color: Color::BLUE,
        position: Default::default(),
        rotation: Default::default(),
    });
    object
        .spawn(
            Transform::from_xyz(0.0, 10.0, 0.0),
            RigidBody::Dynamic,
            &mut commands,
            &mut meshes,
            &mut materials,
        )
        .insert(Player)
        .insert(LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z)
        .add_child(camera);
}

fn player_movement(
    mut player: Query<(&mut Velocity, &Transform), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut velocity, _) in player.iter_mut() {
            velocity.linvel += Vec3::Y * 10.0;
        }
    }
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
}
