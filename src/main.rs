use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use ghost::ghost_movement;
use maze::{Intersection, Maze, Path, HALF_PATH_WIDTH};
use object::GameObject;

use crate::ghost::create_ghost;

mod ghost;
mod maze;
mod object;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Update, (player_movement, ghost_movement))
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

    fn intersection_path<'a>(&self, intersection: &'a Intersection) -> &'a Option<Path> {
        // The catch here is that our notion of forward is the opposite of the intersection's.
        // We say forward is the negative z direction (which is how Bevy does it).
        // The intersection says forward is the positive y direction though, so this may look a bit strange.
        match self {
            Direction::Forward => &intersection.backward,
            Direction::Backward => &intersection.forward,
            Direction::Left => &intersection.left,
            Direction::Right => &intersection.right,
        }
    }
}

#[derive(Clone, Debug, Default, Component)]
pub struct Player {
    current_direction: Direction,
    queued_direction: Option<Direction>,
}

#[derive(Component)]
struct IntersectionComponent(pub Intersection);

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

    const PLAYER_RADIUS: f32 = HALF_PATH_WIDTH - 0.1;

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
            .spawn(Collider::ball(HALF_PATH_WIDTH))
            .insert(Sensor)
            .insert(Transform::from_xyz(
                intersection.coordinates.0,
                0.0,
                intersection.coordinates.1,
            ))
            .insert(GlobalTransform::default())
            .insert(IntersectionComponent(intersection.clone()));
    }

    commands.insert_resource(maze);

    create_ghost(
        &mut commands,
        &mut meshes,
        &mut materials,
        Vec3::new(5.0, HALF_PATH_WIDTH, 20.0),
    );
}

fn can_go_that_way(intersection: &Intersection, direction: Direction) -> bool {
    direction.intersection_path(intersection).is_some()
}

#[allow(clippy::type_complexity)]
fn player_movement(
    mut player: Query<(&mut Player, &mut Velocity, &mut Transform, Entity)>,
    intersections: Query<
        (&IntersectionComponent, Entity),
        (With<IntersectionComponent>, Without<Player>),
    >,
    keyboard_input: Res<Input<KeyCode>>,
    rapier_context: Res<RapierContext>,
) {
    for (mut player, mut velocity, mut transform, entity) in player.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::Down) {
            player.current_direction = player.current_direction.rotate_backward();
        }
        let current_intersection = intersections
            .iter()
            .filter(|(_, intersection_entity)| {
                rapier_context
                    .intersection_pair(entity, *intersection_entity)
                    .unwrap_or(false)
            })
            .map(|(intersection, _)| intersection.0.clone())
            .next();
        if keyboard_input.just_pressed(KeyCode::Left) {
            let new_direction = player.current_direction.rotate_left();
            if let Some(current_intersection) = current_intersection
                .as_ref()
                .filter(|intersection| can_go_that_way(intersection, new_direction))
            {
                player.current_direction = new_direction;
                transform.translation.x = current_intersection.coordinates.0;
                transform.translation.z = current_intersection.coordinates.1;
            } else {
                player.queued_direction = Some(new_direction);
            }
        }
        if keyboard_input.just_pressed(KeyCode::Right) {
            let new_direction = player.current_direction.rotate_right();
            if let Some(current_intersection) = current_intersection
                .as_ref()
                .filter(|intersection| can_go_that_way(intersection, new_direction))
            {
                player.current_direction = new_direction;
                transform.translation.x = current_intersection.coordinates.0;
                transform.translation.z = current_intersection.coordinates.1;
            } else {
                player.queued_direction = Some(new_direction);
            }
        }
        if let Some(current_intersection) = &current_intersection {
            if let Some(queued_direction) = player.queued_direction {
                if can_go_that_way(current_intersection, queued_direction) {
                    player.current_direction = queued_direction;
                    player.queued_direction = None;
                    transform.translation.x = current_intersection.coordinates.0;
                    transform.translation.z = current_intersection.coordinates.1;
                }
            }
        }
        const SPEED: f32 = 3.0;
        velocity.linvel.x = player.current_direction.x_velocity() * SPEED;
        velocity.linvel.z = player.current_direction.z_velocity() * SPEED;
        transform.rotation = player.current_direction.get_rotation();
    }
}
