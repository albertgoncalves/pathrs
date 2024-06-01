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

pub struct Dijkstra<'a, T> {
    nodes: &'a [T],
    weights: Vec<f32>,
}

impl<'a, T: Distance<f32> + Copy> Dijkstra<'a, T> {
    pub fn new(nodes: &'a [T], edges: &'a [(usize, usize)]) -> Self {
        let mut weights = vec![f32::INFINITY; nodes.len() * nodes.len()];

        for (i, j) in edges {
            let weight = nodes[*i].distance(nodes[*j]);
            weights[(i * nodes.len()) + j] = weight;
            weights[(j * nodes.len()) + i] = weight;
        }

        Self { nodes, weights }
    }

    // NOTE: See `https://doc.rust-lang.org/std/collections/binary_heap/index.html`.
    pub fn shortest_path(&self, start: usize, end: usize, counter: &mut usize) -> Vec<usize> {
        let mut costs = vec![f32::INFINITY; self.nodes.len()];
        let mut previous = vec![self.nodes.len(); self.nodes.len()];
        let mut heap = BinaryHeap::with_capacity(self.nodes.len());

        costs[start] = 0.0;
        heap.push(Node { index: start, cost: 0.0 });

        *counter = 0;
        while let Some(node) = heap.pop() {
            *counter += 1;
            if node.index == end {
                break;
            }
            if costs[node.index] < node.cost {
                continue;
            }
            for j in 0..self.nodes.len() {
                if self.weights[(node.index * self.nodes.len()) + j].is_infinite() {
                    continue;
                }
                let cost = node.cost + self.weights[(node.index * self.nodes.len()) + j];
                if cost < costs[j] {
                    heap.push(Node { index: j, cost });
                    previous[j] = node.index;
                    costs[j] = cost;
                }
            }
        }

        let mut path = Vec::with_capacity(self.nodes.len());
        {
            let mut i = end;
            while i != start {
                path.push(i);
                i = previous[i];
            }
        }
        path.push(start);
        path.reverse();
        path
    }
}
