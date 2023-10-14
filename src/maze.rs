use bevy::prelude::{Color, Quat, Vec3};

use crate::object::{GameObject, Mesh, Shape};

/// This is essentially a graph, with the nodes being the intersections and the edges being the paths between them.
#[derive(Clone, Debug, PartialEq)]
pub struct Maze {
    intersections: Vec<Intersection>,
}

/// For simplicity, we'll assume that all intersections are at right angles to each other. This means there are up to four paths leading out of each intersection: left, right, forward and backward.
#[derive(Clone, Debug, PartialEq)]
pub struct Intersection {
    pub left: Option<Path>,
    pub right: Option<Path>,
    pub forward: Option<Path>,
    pub backward: Option<Path>,

    pub coordinates: (f32, f32),
}

impl Intersection {
    pub fn new(coordinates: (f32, f32)) -> Self {
        Self {
            left: None,
            right: None,
            forward: None,
            backward: None,
            coordinates,
        }
    }
    pub fn with_paths(
        left: Option<Path>,
        right: Option<Path>,
        forward: Option<Path>,
        backward: Option<Path>,
        coordinates: (f32, f32),
    ) -> Self {
        Self {
            left,
            right,
            forward,
            backward,
            coordinates,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub end_index: usize,
    pub length: f32,
}

impl Path {
    pub fn new(end_index: usize, length: f32) -> Self {
        Self { end_index, length }
    }
}

impl Maze {
    #[allow(clippy::type_complexity)]
    pub fn new(paths: &[((f32, f32), (f32, f32))]) -> Self {
        let mut intersections: Vec<Intersection> = Vec::new();
        // We need to convert from a list of paths to a list of intersections and the paths connecting them (to form a graph of the maze).
        // We do this with a two stage process: first find all the intersections, then break up each path and sort out which intersections it passes through.

        for (i, path) in paths.iter().enumerate() {
            let (start, end) = path;
            assert!(
                start.0 == end.0 || start.1 == end.1,
                "Paths must be either vertical or horizontal"
            );
            let is_horizontal = start.1 == end.1;
            // We need to create paths for the starts and ends of the path.
            // Because these paths are really edges in a graph, we actually need every path to be connected on both sides for it to register.
            let mut start_intersection_exists = intersections
                .iter()
                .any(|intersection| &intersection.coordinates == start);
            let mut end_intersection_exists = intersections
                .iter()
                .any(|intersection| &intersection.coordinates == end);
            // If we only check forward (i.e. we don't check paths covered in the outer loop), this will prevent us from checking the same pair of paths more than once.
            // We do this since it still covers all pairs, is a lot simpler and (hopefully) is faster to execute (although this isn't much of a concern).
            for other_path in paths.iter().skip(i + 1) {
                let (other_start, other_end) = other_path;
                assert!(
                    other_start.0 == other_end.0 || other_start.1 == other_end.1,
                    "Paths must be either vertical or horizontal"
                );
                let other_is_horizontal = other_start.1 == other_end.1;
                // If they are parallel then they can't intersect, unless they are equal.
                assert!(
                    path != other_path,
                    "Duplicate paths: {:?} and {:?}",
                    path,
                    other_path
                );
                if is_horizontal != other_is_horizontal {
                    // Whichever one is vertical will supply the x coordinate of the intersection, and the horizontal one will supply the y.
                    let vertical_path = if is_horizontal { other_path } else { path };
                    let horizontal_path = if is_horizontal { path } else { other_path };
                    let intersection =
                        Intersection::new((vertical_path.0 .0, horizontal_path.0 .1));
                    if &intersection.coordinates == start {
                        start_intersection_exists = true;
                    }
                    if &intersection.coordinates == end {
                        end_intersection_exists = true;
                    }
                    intersections.push(intersection);
                }
            }

            // This is a little bit messy.
            // This is because we want to add the start and end coordinates in the same order regardless of the direction of the path (since we support paths going 'the wrong way', meaning with a start greater than the end).
            // Our unit tests check that changing the direction of the paths doesn't change the maze, so we have to output the intersections in the same order either way.
            let add_start_if_necessary = |intersections: &mut Vec<Intersection>| {
                if !start_intersection_exists {
                    intersections.push(Intersection::new(*start));
                }
            };
            let add_end_if_necessary = |intersections: &mut Vec<Intersection>| {
                if !end_intersection_exists {
                    intersections.push(Intersection::new(*end));
                }
            };
            let is_going_forward = start.0 < end.0 || start.1 < end.1;
            if is_going_forward {
                add_start_if_necessary(&mut intersections);
                add_end_if_necessary(&mut intersections);
            } else {
                add_end_if_necessary(&mut intersections);
                add_start_if_necessary(&mut intersections);
            }
        }

        for path in paths {
            let (start, end) = path;
            let is_horizontal = start.1 == end.1;
            // We need to be able to handle paths which go backwards (i.e. the start is greater than the end).
            let is_moving_forward = start.0 < end.0 || start.1 < end.1;
            // What we want is a list of all the intersections this path passes through, sorted by distance to the start.
            // We can calculate distance using abs, rather than sqrt, because one of the terms in the distance formula should always be 0 (because the paths are always horizontal or vertical).
            // For example, if the path is vertical, the intersection should have the same x value as the path.
            // This means that (path.x-intersection.x) is 0, and therefore doesn't influence the distance. This leaves us with the distance as sqrt((path.y-intersection.y)^2), which is equal to abs(path.y-intersection.y).
            let mut matching_intersections_with_distances = intersections
                .iter_mut()
                .enumerate()
                .filter(|(_, intersection)| {
                    // Unfortunately, the logic for this line segment intersection is different depending on the direction of the line.
                    if is_moving_forward {
                        if is_horizontal {
                            intersection.coordinates.1 == start.1
                                && intersection.coordinates.0 >= start.0
                                && intersection.coordinates.0 <= end.0
                        } else {
                            intersection.coordinates.0 == start.0
                                && intersection.coordinates.1 >= start.1
                                && intersection.coordinates.1 <= end.1
                        }
                    } else if is_horizontal {
                        intersection.coordinates.1 == start.1
                            && intersection.coordinates.0 >= end.0
                            && intersection.coordinates.0 <= start.0
                    } else {
                        intersection.coordinates.0 == start.0
                            && intersection.coordinates.1 >= end.1
                            && intersection.coordinates.1 <= start.1
                    }
                })
                .map(|(index, intersection)| {
                    (
                        (intersection.coordinates.0 - start.0).abs()
                            + (intersection.coordinates.1 - start.1).abs(),
                        (index, intersection),
                    )
                })
                .collect::<Vec<_>>();
            matching_intersections_with_distances
                .sort_by(|(a_distance, _), (b_distance, _)| a_distance.total_cmp(b_distance));
            // Because of Rust's borrowing rules, we can't actually access different elements of the intersections Vec.
            // However, we need to check the distances of the previous and next elements to be able to compute the lengths of the edges connecting them.
            let distances_and_indexes = matching_intersections_with_distances
                .iter()
                .map(|(distance, (index, _))| (*distance, *index))
                .collect::<Vec<_>>();
            for (i, (distance, (_, intersection))) in
                matching_intersections_with_distances.iter_mut().enumerate()
            {
                let path_to_previous = if i > 0 {
                    Some(Path {
                        end_index: distances_and_indexes[i - 1].1,
                        length: *distance - distances_and_indexes[i - 1].0,
                    })
                } else {
                    None
                };
                let path_to_next = if i + 1 < distances_and_indexes.len() {
                    Some(Path {
                        end_index: distances_and_indexes[i + 1].1,
                        length: distances_and_indexes[i + 1].0 - *distance,
                    })
                } else {
                    None
                };

                if is_moving_forward {
                    if is_horizontal {
                        intersection.left = path_to_previous;
                        intersection.right = path_to_next;
                    } else {
                        intersection.backward = path_to_previous;
                        intersection.forward = path_to_next;
                    }
                } else if is_horizontal {
                    intersection.right = path_to_previous;
                    intersection.left = path_to_next;
                } else {
                    intersection.forward = path_to_previous;
                    intersection.backward = path_to_next;
                }
            }
        }
        Self { intersections }
    }

    pub fn intersections(&self) -> &Vec<Intersection> {
        &self.intersections
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_maze_simple() {
        let maze = Maze::new(&[((0.0, 1.0), (0.0, -1.0)), ((1.0, 0.0), (-1.0, 0.0))]);
        assert_eq!(
            maze.intersections,
            vec![
                Intersection::with_paths(
                    Some(Path::new(4, 1.0)),
                    Some(Path::new(3, 1.0)),
                    Some(Path::new(1, 1.0)),
                    Some(Path::new(2, 1.0)),
                    (0.0, 0.0)
                ),
                Intersection::with_paths(None, None, None, Some(Path::new(0, 1.0)), (0.0, 1.0)),
                Intersection::with_paths(None, None, Some(Path::new(0, 1.0)), None, (0.0, -1.0)),
                Intersection::with_paths(Some(Path::new(0, 1.0)), None, None, None, (1.0, 0.0)),
                Intersection::with_paths(None, Some(Path::new(0, 1.0)), None, None, (-1.0, 0.0)),
            ]
        );
        let maze = Maze::new(&[
            ((0.0, 1.0), (0.0, -1.0)),
            ((1.0, 0.0), (-1.0, 0.0)),
            ((1.0, 1.0), (1.0, -1.0)),
        ]);
        assert_eq!(
            maze.intersections,
            vec![
                Intersection::with_paths(
                    Some(Path::new(4, 1.0)),
                    Some(Path::new(3, 1.0)),
                    Some(Path::new(1, 1.0)),
                    Some(Path::new(2, 1.0)),
                    (0.0, 0.0)
                ),
                Intersection::with_paths(None, None, None, Some(Path::new(0, 1.0)), (0.0, 1.0)),
                Intersection::with_paths(None, None, Some(Path::new(0, 1.0)), None, (0.0, -1.0)),
                Intersection::with_paths(
                    Some(Path::new(0, 1.0)),
                    None,
                    Some(Path::new(5, 1.0)),
                    Some(Path::new(6, 1.0)),
                    (1.0, 0.0)
                ),
                Intersection::with_paths(None, Some(Path::new(0, 1.0)), None, None, (-1.0, 0.0)),
                Intersection::with_paths(None, None, None, Some(Path::new(3, 1.0)), (1.0, 1.0)),
                Intersection::with_paths(None, None, Some(Path::new(3, 1.0)), None, (1.0, -1.0)),
            ]
        );
    }

    #[test]
    fn create_maze_backward_paths() {
        let maze1 = Maze::new(&[((0.0, 1.0), (0.0, -1.0)), ((1.0, 0.0), (-1.0, 0.0))]);
        let maze2 = Maze::new(&[((0.0, -1.0), (0.0, 1.0)), ((-1.0, 0.0), (1.0, 0.0))]);
        assert_eq!(maze1, maze2);
    }
}

impl Maze {
    pub fn create_game_object(&self) -> GameObject {
        const HALF_PATH_WIDTH: f32 = 1.0;
        const PATH_THICKNESS: f32 = 0.01;
        let mut meshes: Vec<Mesh> = Vec::new();
        for intersection in &self.intersections {
            let mut paths: Vec<Path> = Vec::new();
            // By only considering the up and right paths, we simplify the logic a lot.
            // Every path has two intersections, which have the path on opposite edges. Therefore every path will always be either an up or a right of some intersection.
            if let Some(path) = &intersection.right {
                paths.push(path.clone());
            }
            if let Some(path) = &intersection.forward {
                paths.push(path.clone());
            }
            for path in paths {
                let target_intersection = &self.intersections[path.end_index];
                let is_horizontal = intersection.coordinates.1 == target_intersection.coordinates.1;
                let width = if is_horizontal {
                    target_intersection.coordinates.0
                        - intersection.coordinates.0
                        - 2.0 * HALF_PATH_WIDTH
                } else {
                    PATH_THICKNESS
                };
                let depth = if is_horizontal {
                    PATH_THICKNESS
                } else {
                    target_intersection.coordinates.1
                        - intersection.coordinates.1
                        - 2.0 * HALF_PATH_WIDTH
                };
                // Its important to note that the positions are actually teh positions of the centers of the shapes, so we have to add half of the width and depth.
                // The logic for horizontal and vertical paths turns out to be exactly the same for position1.
                let position1 = Vec3::new(
                    intersection.coordinates.0 + HALF_PATH_WIDTH,
                    HALF_PATH_WIDTH,
                    intersection.coordinates.1 + HALF_PATH_WIDTH,
                ) + Vec3::new(width, 0.0, depth) / 2.0;
                let position2 = if is_horizontal {
                    Vec3::new(
                        intersection.coordinates.0 + HALF_PATH_WIDTH,
                        HALF_PATH_WIDTH,
                        intersection.coordinates.1 - HALF_PATH_WIDTH,
                    )
                } else {
                    Vec3::new(
                        intersection.coordinates.0 - HALF_PATH_WIDTH,
                        HALF_PATH_WIDTH,
                        intersection.coordinates.1 + HALF_PATH_WIDTH,
                    )
                } + Vec3::new(width, 0.0, depth) / 2.0;
                meshes.push(Mesh {
                    position: position1,
                    rotation: Quat::default(),
                    color: Color::GRAY,
                    shape: Shape::Box {
                        width,
                        height: HALF_PATH_WIDTH * 2.0,
                        depth,
                    },
                });
                meshes.push(Mesh {
                    position: position2,
                    rotation: Quat::default(),
                    color: Color::GRAY,
                    shape: Shape::Box {
                        width,
                        height: HALF_PATH_WIDTH * 2.0,
                        depth,
                    },
                });
            }
            let mut missing_path_position_width_depth = Vec::new();
            if intersection.left.is_none() {
                missing_path_position_width_depth.push((
                    Vec3::new(
                        intersection.coordinates.0 - HALF_PATH_WIDTH,
                        HALF_PATH_WIDTH,
                        intersection.coordinates.1,
                    ),
                    PATH_THICKNESS,
                    HALF_PATH_WIDTH * 2.0,
                ));
            }
            if intersection.right.is_none() {
                missing_path_position_width_depth.push((
                    Vec3::new(
                        intersection.coordinates.0 + HALF_PATH_WIDTH,
                        HALF_PATH_WIDTH,
                        intersection.coordinates.1,
                    ),
                    PATH_THICKNESS,
                    HALF_PATH_WIDTH * 2.0,
                ));
            }
            if intersection.forward.is_none() {
                missing_path_position_width_depth.push((
                    Vec3::new(
                        intersection.coordinates.0,
                        HALF_PATH_WIDTH,
                        intersection.coordinates.1 + HALF_PATH_WIDTH,
                    ),
                    HALF_PATH_WIDTH * 2.0,
                    PATH_THICKNESS,
                ));
            }
            if intersection.backward.is_none() {
                missing_path_position_width_depth.push((
                    Vec3::new(
                        intersection.coordinates.0,
                        HALF_PATH_WIDTH,
                        intersection.coordinates.1 - HALF_PATH_WIDTH,
                    ),
                    HALF_PATH_WIDTH * 2.0,
                    PATH_THICKNESS,
                ));
            }
            for (position, width, depth) in missing_path_position_width_depth {
                meshes.push(Mesh {
                    position,
                    rotation: Quat::default(),
                    color: Color::GRAY,
                    shape: Shape::Box {
                        width,
                        height: HALF_PATH_WIDTH * 2.0,
                        depth,
                    },
                });
            }
        }
        let mut result = GameObject::default();
        meshes.into_iter().for_each(|mesh| {
            result.add_mesh(mesh);
        });
        result
    }
}
