use crate::{Coords, CoordsFloat, PieceKind};

use super::{mino123::Mino123, tetromino::TetrominoSrs, RotationState, PieceKindTrait};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mino1234 {
    Mino123(Mino123),
    TetrominoSrs(TetrominoSrs),
}

impl PieceKindTrait for Mino1234 {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            Mino1234::Mino123(p) => p.spawn_offsets(),
            Mino1234::TetrominoSrs(p) => p.spawn_offsets(),
        }
    }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            Mino1234::Mino123(p) => p.pivot_offset(rotation_state),
            Mino1234::TetrominoSrs(p) => p.pivot_offset(rotation_state),
        }
    }

    fn asset_name(&self) -> &str {
        match self {
            Mino1234::Mino123(p) => p.asset_name(),
            Mino1234::TetrominoSrs(p) => p.asset_name(),
        }
    }

    fn iter() -> Box<dyn Iterator<Item = PieceKind>> {
        Box::new(<Mino123 as PieceKindTrait>::iter().chain(<TetrominoSrs as PieceKindTrait>::iter()))
    }

    fn n_kinds() -> usize { 11 }
}
