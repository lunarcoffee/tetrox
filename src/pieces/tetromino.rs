use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{Coords, CoordsFloat, PieceKind, kicks::RotationState};

use super::PieceKindTrait;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum TetrominoSrs {
    S,
    Z,
    L,
    J,
    T,
    O,
    I,
}

impl PieceKindTrait for TetrominoSrs {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            TetrominoSrs::S => [(0, -1), (0, 0), (-1, 0), (-1, 1)],
            TetrominoSrs::Z => [(0, 0), (0, 1), (-1, -1), (-1, 0)],
            TetrominoSrs::L => [(0, -1), (0, 0), (0, 1), (-1, 1)],
            TetrominoSrs::J => [(0, -1), (0, 0), (0, 1), (-1, -1)],
            TetrominoSrs::T => [(0, -1), (0, 0), (0, 1), (-1, 0)],
            TetrominoSrs::O => [(0, 0), (0, 1), (-1, 0), (-1, 1)],
            TetrominoSrs::I => [(0, -1), (0, 0), (0, 1), (0, 2)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            TetrominoSrs::S => (1, CoordsFloat::zero()),
            TetrominoSrs::Z => (0, CoordsFloat::zero()),
            TetrominoSrs::L => (1, CoordsFloat::zero()),
            TetrominoSrs::J => (1, CoordsFloat::zero()),
            TetrominoSrs::T => (1, CoordsFloat::zero()),
            TetrominoSrs::O => (2, TetrominoSrs::I.pivot_offset(rotation_state).1),
            TetrominoSrs::I => (
                1,
                match rotation_state {
                    RotationState::Initial => CoordsFloat(0.5, 0.5),
                    RotationState::Cw => CoordsFloat(0.5, -0.5),
                    RotationState::Flipped => CoordsFloat(-0.5, -0.5),
                    RotationState::Ccw => CoordsFloat(-0.5, 0.5),
                },
            ),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            TetrominoSrs::S => "s",
            TetrominoSrs::Z => "z",
            TetrominoSrs::L => "l",
            TetrominoSrs::J => "j",
            TetrominoSrs::T => "t",
            TetrominoSrs::O => "o",
            TetrominoSrs::I => "i",
        }
    }

    fn asset_name(&self) -> &str {
        self.display_name()
    }

    fn iter() -> Box<dyn Iterator<Item = PieceKind>> {
        Box::new(<Self as IntoEnumIterator>::iter().map(|p| PieceKind::TetrominoSrs(p)))
    }

    fn n_kinds() -> usize { 7 }
}

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum TetrominoAsc {
    S,
    Z,
    L,
    J,
    T,
    O,
    I,
}

impl TetrominoAsc {
    fn to_srs(&self) -> TetrominoSrs {
        match self {
            TetrominoAsc::S => TetrominoSrs::S,
            TetrominoAsc::Z => TetrominoSrs::Z,
            TetrominoAsc::L => TetrominoSrs::L,
            TetrominoAsc::J => TetrominoSrs::J,
            TetrominoAsc::T => TetrominoSrs::T,
            TetrominoAsc::O => TetrominoSrs::O,
            TetrominoAsc::I => TetrominoSrs::I,
        }
    }
}

impl PieceKindTrait for TetrominoAsc {
    fn spawn_offsets(&self) -> Vec<Coords> { self.to_srs().spawn_offsets() }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            TetrominoAsc::O => (0, CoordsFloat::zero()),
            TetrominoAsc::I => (1, CoordsFloat::zero()),
            _ => self.to_srs().pivot_offset(rotation_state),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            TetrominoAsc::S => "s",
            TetrominoAsc::Z => "z",
            TetrominoAsc::L => "l",
            TetrominoAsc::J => "j",
            TetrominoAsc::T => "t",
            TetrominoAsc::O => "o",
            TetrominoAsc::I => "i",
        }
    }

    fn asset_name(&self) -> &str {
        self.display_name()
    }

    fn iter() -> Box<dyn Iterator<Item = PieceKind>> {
        Box::new(<Self as IntoEnumIterator>::iter().map(|p| PieceKind::TetrominoAsc(p)))
    }

    fn n_kinds() -> usize { 7 }
}
