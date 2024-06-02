use crate::math::Distance;
use std::cmp;
use std::collections::{BinaryHeap, VecDeque};

#[derive(Copy, Clone, PartialEq)]
struct Node<T> {
    index: usize,
    cost: T,
    heuristic: T,
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
        (other.cost + other.heuristic)
            .total_cmp(&(self.cost + self.heuristic))
            .then_with(|| other.index.cmp(&self.index))
    }
}

// NOTE: See `https://doc.rust-lang.org/std/collections/binary_heap/index.html`.
pub fn shortest_path<T: Distance<f32> + Copy>(
    nodes: &[T],
    weights: &[f32],
    start: usize,
    end: usize,
    counter: &mut usize,
) -> VecDeque<usize> {
    let mut costs = vec![f32::INFINITY; nodes.len()];
    costs[start] = 0.0;

    let heuristics: Vec<f32> = nodes.iter().map(|node| node.distance(nodes[end])).collect();

    let mut heap = BinaryHeap::with_capacity(nodes.len());
    heap.push(Node {
        index: start,
        cost: costs[start],
        heuristic: heuristics[start],
    });

    *counter = 0;
    let mut previous = vec![nodes.len(); nodes.len()];
    while let Some(node) = heap.pop() {
        *counter += 1;
        if node.index == end {
            break;
        }
        if costs[node.index] < node.cost {
            continue;
        }
        for j in 0..nodes.len() {
            if weights[(node.index * nodes.len()) + j].is_infinite() {
                continue;
            }
            let cost = node.cost + weights[(node.index * nodes.len()) + j];
            if cost < costs[j] {
                heap.push(Node {
                    index: j,
                    cost,
                    heuristic: heuristics[j],
                });
                previous[j] = node.index;
                costs[j] = cost;
            }
        }
    }

    let mut path = VecDeque::with_capacity(nodes.len());
    {
        let mut i = end;
        while i != start {
            path.push_front(i);
            i = previous[i];
        }
    }
    path.push_front(start);
    path
}
