use pad::Position;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Rect {
    pub position: Position,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub fn new(position: Position, width: usize, height: usize) -> Self {
        Self { position, width, height }
    }
}