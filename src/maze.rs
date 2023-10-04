/// This is essentially a graph, with the nodes being the intersections and the edges being the paths between them.
pub struct Maze {
    intersections: Vec<Intersection>,
}

/// For simplicity, we'll assume that all intersections are at right angles to each other. This means there are up to four paths leading out of each intersection: left, right, forward and backward.
pub struct Intersection {
    left: Option<usize>,
    right: Option<usize>,
    forward: Option<usize>,
    backward: Option<usize>,

    coordinates: (i32, i32),
}

pub struct Path {
    end_index: usize,
    length: f32,
}
