use hashbrown::HashSet;
use rand::{distributions::Alphanumeric, Rng};
use std::iter::FromIterator;

pub mod captcha;

pub fn vec_to_set<T: Clone + Eq + std::hash::Hash>(data: &[T]) -> HashSet<T> {
    HashSet::from_iter(data.iter().cloned())
}

pub fn gen_token(l: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(l)
        .collect::<String>()
}
