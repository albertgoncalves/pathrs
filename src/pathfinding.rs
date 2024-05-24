use crate::math::Distance;
use std::cmp;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, PartialEq)]
struct Node<T> {
    index: usize,
    cost: T,
}

// NOTE: See `https://stackoverflow.com/questions/39949939/how-can-i-implement-a-min-heap-of-f64-with-rusts-binaryheap`.
impl Eq for Node<f32> {}

impl PartialOrd for Node<f32> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Node<f32> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        // NOTE: See `https://doc.rust-lang.org/std/primitive.f32.html#method.total_cmp`.
        other.cost.total_cmp(&self.cost).then_with(|| other.index.cmp(&self.index))
    }
}

#[allow(clippy::needless_range_loop)]
pub fn dijkstra<const N: usize, T: Distance<f32> + Copy>(
    weights: &mut [[f32; N]; N],
    nodes: &[T],
    edges: &[(usize, usize)],
    start: usize,
    end: usize,
) -> Vec<usize> {
    if start == end {
        return vec![start];
    }

    let n = nodes.len();

    for i in 0..n {
        for j in 0..n {
            if i == j {
                weights[i][j] = 0.0;
            } else {
                weights[i][j] = f32::INFINITY;
            }
        }
    }

    for (i, j) in edges {
        let weight = nodes[*i].distance(nodes[*j]);
        weights[*i][*j] = weight;
        weights[*j][*i] = weight;
    }

    let mut costs: Vec<f32> = vec![f32::INFINITY; n];
    let mut path: Vec<usize> = vec![n; n];

    // NOTE: See `https://doc.rust-lang.org/std/collections/binary_heap/index.html`.
    let mut heap: BinaryHeap<Node<f32>> = BinaryHeap::new();
    costs[start] = 0.0;

    heap.push(Node { index: start, cost: 0.0 });
    while let Some(node) = heap.pop() {
        if node.index == end {
            break;
        }
        if costs[node.index] < node.cost {
            continue;
        }
        for i in 0..n {
            if node.index == i {
                continue;
            }
            if weights[node.index][i].is_infinite() {
                continue;
            }
            let cost = node.cost + weights[node.index][i];
            if cost < costs[i] {
                heap.push(Node { index: i, cost });
                path[i] = node.index;
                costs[i] = cost;
            }
        }
    }

    let mut solution = vec![];
    let mut i = end;
    loop {
        solution.push(i);
        i = path[i];
        if i == start {
            break;
        }
    }
    solution.push(start);
    solution.reverse();
    solution
}
