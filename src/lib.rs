pub mod field;
pub mod tetromino;

use std::ops;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Coords(pub i32, pub i32);

impl Coords {
    pub fn to_coords_float(self) -> CoordsFloat { CoordsFloat(self.0 as f64, self.1 as f64) }
}

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

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CoordsFloat(pub f64, pub f64);

impl CoordsFloat {
    pub fn zero() -> CoordsFloat { CoordsFloat(0., 0.) }

    pub fn to_coords(self) -> Coords { Coords(self.0 as i32, self.1 as i32) }
}

impl ops::Add for CoordsFloat {
    type Output = CoordsFloat;

    fn add(self, CoordsFloat(row2, col2): Self) -> Self::Output { CoordsFloat(self.0 + row2, self.1 + col2) }
}

impl ops::Sub for CoordsFloat {
    type Output = CoordsFloat;

    fn sub(self, CoordsFloat(row2, col2): Self) -> Self::Output { CoordsFloat(self.0 - row2, self.1 - col2) }
}

pub trait PieceKind: Copy {
    // coords of the squares composing the piece relative to the spawn coords
    fn spawn_offsets(&self) -> Vec<Coords>;

    // index of the rotation pivot of the piece with a possibly zero offset
    // pieces like the i tetromino have apparent pivots that intersect
    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat);

    fn asset_name(&self) -> &str;
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
