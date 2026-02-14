use std::ops::Deref;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ArenaBox<T: 'static>(&'static T);

impl<T: 'static> ArenaBox<T> {
  #[must_use]
  pub fn new(reference: &'static T) -> Self {
    Self(reference)
  }

  #[must_use]
  pub fn as_ref(self) -> &'static T {
    self.0
  }
}

impl<T: 'static> Deref for ArenaBox<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    self.0
  }
}
