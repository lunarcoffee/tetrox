use crate::{kicks::RotationState, Coords, CoordsFloat};

use self::{
    mino123::Mino123,
    mino1234::Mino1234,
    pentomino::Pentomino,
    tetromino::{TetrominoAsc, TetrominoSrs},
};

pub mod mino123;
pub mod mino1234;
pub mod pentomino;
pub mod tetromino;

pub trait PieceKindTrait {
    // coords of the squares composing the piece relative to the spawn coords
    fn spawn_offsets(&self) -> Vec<Coords>;

    // index of the rotation pivot of the piece with a possibly zero offset
    // pieces like the i tetromino have apparent pivots that intersect
    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat);

    fn display_name(&self) -> &str;

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
    Pentomino(Pentomino),
}

// generate match statement over all `PieceKind`s that calls a method, optionally with arguments
macro_rules! gen_piece_kind_match {
    ($self:ident, $method:ident $(,)? $($arg:expr),*) => {
        match $self {
            PieceKind::TetrominoSrs(p) => p.$method($($arg,)*),
            PieceKind::TetrominoAsc(p) => p.$method($($arg,)*),
            PieceKind::Mino123(p) => p.$method($($arg,)*),
            PieceKind::Mino1234(p) => p.$method($($arg,)*),
            PieceKind::Pentomino(p) => p.$method($($arg,)*),
        }
    }
}

// same as above but for associated functions
macro_rules! gen_piece_kind_match_associated {
    ($self:ident, $method:ident) => {
        match $self {
            PieceKind::TetrominoSrs(_) => TetrominoSrs::$method(),
            PieceKind::TetrominoAsc(_) => TetrominoAsc::$method(),
            PieceKind::Mino123(_) => Mino123::$method(),
            PieceKind::Mino1234(_) => Mino1234::$method(),
            PieceKind::Pentomino(_) => Pentomino::$method(),
        }
    };
}

impl PieceKind {
    pub fn spawn_offsets(&self) -> Vec<Coords> { gen_piece_kind_match!(self, spawn_offsets) }

    pub fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        gen_piece_kind_match!(self, pivot_offset, rotation_state)
    }

    pub fn display_name(&self) -> &str { gen_piece_kind_match!(self, display_name) }

    pub fn asset_name(&self) -> &str { gen_piece_kind_match!(self, asset_name) }

    pub fn iter(&self) -> Box<dyn Iterator<Item = PieceKind>> { gen_piece_kind_match_associated!(self, iter) }
}

// calculate the correct pivot offset based on the current rotation state and an initial offset
fn make_pivot_offset(rotation_state: RotationState, rows: f64, cols: f64) -> CoordsFloat {
    match rotation_state {
        RotationState::Initial => CoordsFloat(rows, cols),
        RotationState::Cw => CoordsFloat(rows, -cols),
        RotationState::Flipped => CoordsFloat(-rows, -cols),
        RotationState::Ccw => CoordsFloat(-rows, cols),
    }
}
