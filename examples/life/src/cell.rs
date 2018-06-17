#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
  Alive,
  Dead,
}

impl Cell {
  pub fn tick(self, neighbors: u8) -> Cell {
    use Cell::*;
    match (self, neighbors) {
      (Alive, 0) => Dead,
      (Alive, 1) => Dead,
      (Alive, 2) => Alive,
      (Alive, 3) => Alive,
      (Alive, _) => Dead,
      (Dead,  0) => Dead,
      (Dead,  1) => Dead,
      (Dead,  2) => Dead,
      (Dead,  3) => Alive,
      (Dead,  _) => Dead,
    }
  }
}

