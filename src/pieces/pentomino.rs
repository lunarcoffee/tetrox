use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{kicks::RotationState, Coords, CoordsFloat, PieceKind};

use super::PieceKindTrait;

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum Pentomino {
    I,
    F,
    R,
    J,
    L,
    Q,
    P,
    N,
    Zeta,
    T,
    U,
    V,
    W,
    X,
    Y,
    Lambda,
    S,
    Z,
}

impl PieceKindTrait for Pentomino {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            Pentomino::I => [(0, -2), (0, -1), (0, 0), (0, 1), (0, 2)],
            Pentomino::F => [(0, 0), (-1, -1), (-1, 0), (-1, 1), (-2, -1)],
            Pentomino::R => [(0, 0), (-1, -1), (-1, 0), (-1, 1), (-2, 1)],
            Pentomino::J => [(0, -1), (0, 0), (0, 1), (0, 2), (-1, -1)],
            Pentomino::L => [(0, -1), (0, 0), (0, 1), (0, 2), (-1, 2)],
            Pentomino::Q => [(0, -1), (0, 0), (0, 1), (-1, 0), (-1, 1)],
            Pentomino::P => [(0, -1), (0, 0), (0, 1), (-1, -1), (-1, 0)],
            Pentomino::N => [(0, 0), (0, 1), (0, 2), (-1, -1), (-1, 0)],
            Pentomino::Zeta => [(0, -1), (0, 0), (0, 1), (-1, 1), (-1, 2)],
            Pentomino::T => [(0, -1), (0, 0), (0, 1), (-1, 0), (-2, 0)],
            Pentomino::U => [(0, -1), (0, 0), (0, 1), (-1, -1), (-1, 1)],
            Pentomino::V => [(0, -1), (0, 0), (0, 1), (-1, 1), (-2, 1)],
            Pentomino::W => [(0, -1), (0, 0), (-1, 0), (-1, 1), (-2, 1)],
            Pentomino::X => [(0, 0), (-1, -1), (-1, 0), (-1, 1), (-2, 0)],
            Pentomino::Y => [(0, -1), (0, 0), (0, 1), (0, 2), (-1, 1)],
            Pentomino::Lambda => [(0, -1), (0, 0), (0, 1), (0, 2), (-1, 0)],
            Pentomino::S => [(0, 1), (-1, -1), (-1, 0), (-1, 1), (-2, -1)],
            Pentomino::Z => [(0, -1), (-1, -1), (-1, 0), (-1, 1), (-2, 1)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            Pentomino::N => (0, CoordsFloat::zero()),
            Pentomino::J
            | Pentomino::Q
            | Pentomino::P
            | Pentomino::Zeta
            | Pentomino::T
            | Pentomino::U
            | Pentomino::Lambda => (1, CoordsFloat::zero()),
            Pentomino::I
            | Pentomino::F
            | Pentomino::R
            | Pentomino::L
            | Pentomino::W
            | Pentomino::X
            | Pentomino::Y
            | Pentomino::S
            | Pentomino::Z => (2, CoordsFloat::zero()),
            Pentomino::V => (
                2,
                match rotation_state {
                    RotationState::Initial => CoordsFloat(-1.0, -1.0),
                    RotationState::Cw => CoordsFloat(-1.0, 1.0),
                    RotationState::Flipped => CoordsFloat(1.0, 1.0),
                    RotationState::Ccw => CoordsFloat(1.0, -1.0),
                },
            ),
        }
    }

    fn display_name(&self) -> &str {
        match self {
            Pentomino::I => "i",
            Pentomino::F => "f",
            Pentomino::R => "r",
            Pentomino::J => "j",
            Pentomino::L => "l",
            Pentomino::Q => "q",
            Pentomino::P => "p",
            Pentomino::N => "n",
            Pentomino::Zeta => "ζ",
            Pentomino::T => "t",
            Pentomino::U => "u",
            Pentomino::V => "v",
            Pentomino::W => "w",
            Pentomino::X => "x",
            Pentomino::Y => "y",
            Pentomino::Lambda => "λ",
            Pentomino::S => "s",
            Pentomino::Z => "z",
        }
    }

    fn asset_name(&self) -> &str {
        match self {
            Pentomino::I | Pentomino::U => "i",
            Pentomino::F | Pentomino::J | Pentomino::N | Pentomino::V | Pentomino::Lambda => "j",
            Pentomino::R | Pentomino::L | Pentomino::Zeta | Pentomino::Y => "l",
            Pentomino::Q | Pentomino::S => "s",
            Pentomino::P | Pentomino::Z => "z",
            Pentomino::T | Pentomino::W => "t",
            Pentomino::X => "o",
        }
    }

    fn iter() -> Box<dyn Iterator<Item = PieceKind>> {
        Box::new(<Self as IntoEnumIterator>::iter().map(|p| PieceKind::Pentomino(p)))
    }

    fn n_kinds() -> usize { 18 }
}
