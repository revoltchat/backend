use hashbrown::HashSet;
use std::iter::FromIterator;

pub fn vec_to_set<T: Clone + Eq + std::hash::Hash>(data: &Vec<T>) -> HashSet<T> {
    HashSet::from_iter(data.iter().cloned())
}
