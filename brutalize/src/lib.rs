use std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::{hash_map, BinaryHeap, HashMap},
    hash::Hash,
    ops::Add,
};

pub enum Transition<S: State> {
    Indeterminate(S),
    Success,
}

pub trait State: Eq + Hash + PartialEq + Sized {
    type Data;
    type Action;
    type Transitions: IntoIterator<Item = (Self::Action, Transition<Self>)>;
    type Heuristic: Ord + Add<usize, Output = Self::Heuristic>;

    fn transitions(&self, data: &Self::Data) -> Self::Transitions;
    fn heuristic(&self, data: &Self::Data) -> Self::Heuristic;
}

#[derive(Eq, PartialEq)]
struct Node<S: State> {
    state: S,
    distance: usize,
    estimate: S::Heuristic,
    index: usize,
}

impl<S: State> PartialOrd for Node<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: State> Ord for Node<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.estimate.cmp(&self.estimate)
    }
}

pub fn solve<S: State>(initial_state: S, data: &S::Data) -> Option<Vec<S::Action>> {
    let mut states = HashMap::new();
    let mut parents = Vec::new();
    let mut queue = BinaryHeap::<Node<S>>::new();

    // Insert initial state
    let initial_transitions = initial_state.transitions(data);
    states.insert(initial_state, ());

    // Add transitions from initial state
    for (action, transition) in initial_transitions {
        match transition {
            Transition::Indeterminate(state) => {
                parents.push((0, action));

                let estimate = state.heuristic(data) + 1;
                queue.push(Node {
                    state,
                    distance: 1,
                    estimate,
                    index: parents.len(),
                });
            }
            Transition::Success => return Some(vec![action]),
        }
    }

    // Pop states in priority order until empty
    while let Some(parent_node) = queue.pop() {
        if let hash_map::Entry::Vacant(vacant) = states.entry(parent_node.state) {
            for (action, transition) in vacant.key().transitions(data) {
                match transition {
                    Transition::Indeterminate(state) => {
                        parents.push((parent_node.index, action));

                        let estimate = state.heuristic(data) + (parent_node.distance + 1);
                        queue.push(Node {
                            state,
                            distance: parent_node.distance + 1,
                            estimate,
                            index: parents.len(),
                        });
                    }
                    Transition::Success => {
                        let mut result_actions = vec![action];
                        let mut current_index = parent_node.index;
                        while current_index != 0 {
                            let (next_index, action) = parents.swap_remove(current_index - 1);
                            result_actions.push(action);
                            current_index = next_index;
                        }
                        result_actions.reverse();
                        return Some(result_actions);
                    }
                }
            }
            vacant.insert(());
        }
    }

    None
}
