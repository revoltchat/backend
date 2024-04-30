/// Owned, Ref or None Value
#[derive(Clone)]
pub enum Value<'a, T> {
    Owned(T),
    Ref(&'a T),
    None,
}

impl<'a, T> Value<'a, T> {
    /// Check whether this Value exists
    pub fn has(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Get this Value as an Option Ref
    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Owned(t) => Some(t),
            Self::Ref(t) => Some(t),
            Self::None => None,
        }
    }

    /// Set owned value
    pub fn set(&mut self, t: T) {
        *self = Value::Owned(t);
    }

    /// Set referential value
    pub fn set_ref(&mut self, t: &'a T) {
        *self = Value::Ref(t);
    }

    /// Clear current value
    pub fn clear(&mut self) {
        *self = Value::None;
    }
}
