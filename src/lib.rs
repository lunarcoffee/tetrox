pub mod field;
pub mod tetromino;

use std::ops;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Coords(i32, i32);

impl ops::Add for Coords {
    type Output = Coords;

    fn add(self, Coords(row2, col2): Self) -> Self::Output { Coords(self.0 + row2, self.1 + col2) }
}

impl ops::Sub for Coords {
    type Output = Coords;

    fn sub(self, Coords(row2, col2): Self) -> Self::Output { Coords(self.0 - row2, self.1 - col2) }
}

impl ops::Neg for Coords {
    type Output = Coords;

    fn neg(self) -> Self::Output { Coords(-self.0, -self.1) }
}

pub trait PieceKind: Copy {
    // coords of the squares composing the piece relative to the spawn coords
    fn spawn_offsets(&self) -> Vec<Coords>;
}

pub trait Bag<P: PieceKind> {
    fn next(&mut self) -> P;
    fn peek(&mut self) -> Box<dyn Iterator<Item = &P> + '_>;

    fn lookahead(&self) -> usize;
}

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum RotationState {
    Initial,
    Cw,
    Flipped,
    Ccw,
}

impl RotationState {
    fn next_cw(&self) -> RotationState { RotationState::from_i32((self.to_i32().unwrap() + 1) % 4).unwrap() }

    fn next_ccw(&self) -> RotationState { RotationState::from_i32((self.to_i32().unwrap() + 3) % 4).unwrap() }
}

pub trait KickTable<P: PieceKind> {
    fn rotate_cw(&self, piece: P, rotation_state: RotationState) -> Vec<Coords>;
    fn rotate_ccw(&self, piece: P, rotation_state: RotationState) -> Vec<Coords>;
}

pub trait KickTable180<P: PieceKind>: KickTable<P> {
    fn rotate_180(&self, piece: P, rotation_state: RotationState) -> Vec<Coords>;
}
