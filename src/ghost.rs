use std::collections::HashMap;

use bevy::prelude::{
    Assets, Color, Commands, Component, Entity, Quat, Query, Res, ResMut, StandardMaterial,
    Transform, Vec3, Without,
};
use bevy_rapier3d::prelude::{LockedAxes, RigidBody, Velocity};

use crate::{
    maze::{Maze, HALF_PATH_WIDTH},
    object::{GameObject, Mesh, Shape},
    Player,
};

#[derive(Clone, Debug, Default, Component)]
pub struct Ghost;

pub fn create_ghost(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<bevy::prelude::Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    initial_position: Vec3,
) -> Entity {
    let mut game_object = GameObject::default();
    game_object.add_mesh(Mesh {
        color: Color::BLUE,
        position: Vec3::default(),
        rotation: Quat::default(),
        shape: Shape::Sphere {
            radius: HALF_PATH_WIDTH - 0.1,
        },
    });
    game_object
        .spawn(
            Transform::from_translation(initial_position),
            RigidBody::Dynamic,
            commands,
            meshes,
            materials,
        )
        .insert(Ghost)
        .insert(LockedAxes::ROTATION_LOCKED)
        .id()
}

/// Finds the indices of the two intersections which this position is between.
fn find_path(position: (f32, f32), maze: &Maze) -> Option<(usize, usize)> {
    for (intersection_index, intersection) in maze.intersections().iter().enumerate() {
        if intersection.coordinates.0 == position.0 {
            if intersection.coordinates.1 < position.1 {
                let distance = position.1 - intersection.coordinates.1;
                if let Some(forward_path) = intersection
                    .forward
                    .as_ref()
                    .filter(|forward_path| distance < forward_path.length)
                {
                    return Some((intersection_index, forward_path.end_index));
                }
            } else if intersection.coordinates.1 > position.1 {
                let distance = intersection.coordinates.1 - position.1;
                if let Some(backward_path) = intersection
                    .backward
                    .as_ref()
                    .filter(|backward_path| distance < backward_path.length)
                {
                    return Some((intersection_index, backward_path.end_index));
                }
            } else {
                return Some((intersection_index, intersection_index));
            }
        } else if intersection.coordinates.1 == position.1 {
            if intersection.coordinates.0 < position.0 {
                let distance = position.0 - intersection.coordinates.0;
                if let Some(right_path) = intersection
                    .right
                    .as_ref()
                    .filter(|right_path| distance < right_path.length)
                {
                    return Some((intersection_index, right_path.end_index));
                }
            } else if intersection.coordinates.0 > position.0 {
                let distance = intersection.coordinates.0 - position.0;
                if let Some(left_path) = intersection
                    .left
                    .as_ref()
                    .filter(|left_path| distance < left_path.length)
                {
                    return Some((intersection_index, left_path.end_index));
                }
            } else {
                return Some((intersection_index, intersection_index));
            }
        }
    }
    None
}

/// We need to find the distance between two points so often that I made this little utility function.
/// TODO: Maybe this should be moved to some sort of math utility file?
fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

pub fn find_shortest_path(
    player_position: (f32, f32),
    current_ghost_position: (f32, f32),
    maze: &Maze,
) -> Vec<usize> {
    let player_path = find_path(player_position, maze).expect("Player not on a path");
    let ghost_path = find_path(current_ghost_position, maze).expect("Ghost not on a path");
    if player_path == ghost_path {
        // We are already on the right path, so we don't actually have to do anythign but chase the player down by moving in their direction.
        return vec![];
    }
    let mut tried_indices_to_distances = HashMap::new();
    let mut potential_paths = {
        let initial_ghost_intersections = (
            &maze.intersections()[ghost_path.0],
            &maze.intersections()[ghost_path.1],
        );
        let initial_ghost_distances = (
            distance(
                initial_ghost_intersections.0.coordinates,
                current_ghost_position,
            ),
            distance(
                initial_ghost_intersections.1.coordinates,
                current_ghost_position,
            ),
        );
        if ghost_path.0 != ghost_path.1 {
            vec![
                (initial_ghost_distances.0, vec![ghost_path.0]),
                (initial_ghost_distances.1, vec![ghost_path.1]),
            ]
        } else {
            vec![(initial_ghost_distances.0, vec![ghost_path.0])]
        }
    };
    let mut completed_paths = Vec::new();
    while !potential_paths.is_empty() {
        // Will terminate if we run out of paths to try.
        let mut new_potential_paths = Vec::new();
        for (accumulating_distance, potential_path) in potential_paths.iter() {
            let current_index = *potential_path.last().unwrap();
            // We don't want to try the same index twice unless we have found a faster way of getting there.
            // If we have, it would already have been registered by the code which created this potential path.
            if let Some(distance) = tried_indices_to_distances.get(&current_index) {
                if accumulating_distance > distance {
                    continue;
                }
            } else {
                tried_indices_to_distances.insert(current_index, *accumulating_distance);
            }
            let current_intersection = &maze.intersections()[current_index];
            let joining_paths = current_intersection
                .forward
                .iter()
                .chain(current_intersection.backward.iter())
                .chain(current_intersection.left.iter())
                .chain(current_intersection.right.iter());
            for joining_path in joining_paths {
                let new_distance = accumulating_distance + joining_path.length;
                // We don't want to add a path if it is covering an index which can be reached by a faster route.
                if tried_indices_to_distances
                    .get(&joining_path.end_index)
                    .filter(|other_distance| **other_distance <= new_distance)
                    .is_none()
                {
                    let mut new_path = potential_path.clone();
                    new_path.push(joining_path.end_index);
                    if joining_path.end_index == player_path.0
                        || joining_path.end_index == player_path.1
                    {
                        // We have found a path which will lead us to the player, so we can stop looking.
                        let joined_intersection = &maze.intersections()[joining_path.end_index];
                        completed_paths.push((
                            new_distance
                                + distance(joined_intersection.coordinates, player_position),
                            new_path,
                        ));
                    } else {
                        new_potential_paths.push((new_distance, new_path));
                    }
                }
            }
        }
        potential_paths = new_potential_paths;
    }

    let shortest_path = completed_paths
        .into_iter()
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .expect("No path to player found")
        .1;
    shortest_path
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_path() {
        let maze = Maze::new(&[((1.0, 0.0), (-1.0, 0.0)), ((0.0, 1.0), (0.0, -1.0))]);
        let left_path = find_path((-0.5, 0.0), &maze);
        assert_eq!(left_path, Some((0, 1)));
        let right_path = find_path((0.5, 0.0), &maze);
        assert_eq!(right_path, Some((0, 2)));
        let forward_path = find_path((0.0, 0.5), &maze);
        assert_eq!(forward_path, Some((0, 4)));
        let backward_path = find_path((0.0, -0.5), &maze);
        assert_eq!(backward_path, Some((0, 3)));
    }

    #[test]
    fn test_find_shortest_path() {
        let maze = Maze::new(&[
            ((1.0, 0.0), (-1.0, 0.0)),
            ((0.0, 1.0), (0.0, -1.0)),
            ((1.0, 1.0), (1.0, -1.0)),
            ((1.0, 1.0), (-1.0, 1.0)),
            ((-1.0, 1.0), (-1.0, -1.0)),
            ((-1.0, -1.0), (1.0, -1.0)),
        ]);
        let shortest_path = find_shortest_path((0.0, 0.0), (0.0, 0.0), &maze);
        assert!(shortest_path.is_empty());
        let shortest_path = find_shortest_path((0.0, 0.0), (-1.0, 0.5), &maze);
        // It's a bit nasty, but I don't want to have to worry about what indices the intersections are at.
        assert_eq!(
            maze.intersections()[shortest_path[0]].coordinates,
            (-1.0, 0.0)
        );
        assert_eq!(
            maze.intersections()[shortest_path[1]].coordinates,
            (0.0, 0.0)
        );
    }
}

pub fn ghost_movement(
    player: Query<(&Transform, &Player)>,
    mut ghosts: Query<(&Ghost, &Transform, &mut Velocity), Without<Player>>,
    maze: Res<Maze>,
) {
    let player = player.get_single().unwrap();
    for (ghost, ghost_transform, mut ghost_velocity) in ghosts.iter_mut() {
        ghost_velocity.linvel = Vec3::new(1.0, 0.0, 0.0);
    }
}
