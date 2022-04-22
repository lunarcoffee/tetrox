use crate::{Coords, CoordsFloat, kicks::RotationState};

use self::{tetromino::{TetrominoSrs, TetrominoAsc}, mino123::Mino123, mino1234::Mino1234};

pub mod mino123;
pub mod tetromino;
pub mod mino1234;

pub trait PieceKindTrait {
    // coords of the squares composing the piece relative to the spawn coords
    fn spawn_offsets(&self) -> Vec<Coords>;

    // index of the rotation pivot of the piece with a possibly zero offset
    // pieces like the i tetromino have apparent pivots that intersect
    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat);

    fn asset_name(&self) -> &str;

    // iterator through all piece kinds
    fn iter() -> Box<dyn Iterator<Item = PieceKind>>;

    fn n_kinds() -> usize;
}

// a piece kind (e.g. t tetromino (srs), domino, l tromino)
// not a trait to avoid trait objects as this type is used in relatively large numbers
#[derive(Copy, Clone, Debug)]
pub enum PieceKind {
    TetrominoSrs(TetrominoSrs),
    TetrominoAsc(TetrominoAsc),
    Mino123(Mino123),
    Mino1234(Mino1234),
}

// generate match statement over all `PieceKind`s that calls a method, optionally with arguments
macro_rules! gen_piece_kind_match {
    ($self:ident, $method:ident $(,)? $($arg:expr),*) => { match $self {
        PieceKind::TetrominoSrs(p) => p.$method($($arg,)*),
        PieceKind::TetrominoAsc(p) => p.$method($($arg,)*),
        PieceKind::Mino123(p) => p.$method($($arg,)*),
        PieceKind::Mino1234(p) => p.$method($($arg,)*),
    } }
}

impl PieceKind {
    pub fn spawn_offsets(&self) -> Vec<Coords> { gen_piece_kind_match!(self, spawn_offsets) }

    pub fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        gen_piece_kind_match!(self, pivot_offset, rotation_state)
    }

    pub fn asset_name(&self) -> &str { gen_piece_kind_match!(self, asset_name) }
}
