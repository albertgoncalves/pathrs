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

pub fn init<const N: usize, T: Distance<f32> + Copy>(
    nodes: &[T],
    edges: &[(usize, usize)],
    weights: &mut [[f32; N]; N],
) {
    for (i, row) in weights.iter_mut().enumerate().take(N) {
        for (j, cell) in row.iter_mut().enumerate().take(N) {
            if i == j {
                *cell = 0.0;
            } else {
                *cell = f32::INFINITY;
            }
        }
    }

    for (i, j) in edges {
        let weight = nodes[*i].distance(nodes[*j]);
        weights[*i][*j] = weight;
        weights[*j][*i] = weight;
    }
}

pub fn dijkstra<const N: usize>(
    weights: &[[f32; N]; N],
    start: usize,
    end: usize,
    path: &mut [usize; N],
) -> usize {
    let mut costs: [f32; N] = [f32::INFINITY; N];
    let mut previous: [usize; N] = [N; N];

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
        for i in 0..N {
            if node.index == i {
                continue;
            }
            if weights[node.index][i].is_infinite() {
                continue;
            }
            let cost = node.cost + weights[node.index][i];
            if cost < costs[i] {
                heap.push(Node { index: i, cost });
                previous[i] = node.index;
                costs[i] = cost;
            }
        }
    }

    let mut i = end;
    let mut j = 0;
    while i != start {
        path[j] = i;
        j += 1;
        i = previous[i];
    }
    path[j] = start;
    j += 1;

    path[..j].reverse();

    j
}
