use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{field::DefaultField, Coords, CoordsFloat, PieceKind};

use super::{PieceKindTrait, RotationState};

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum Mino123 {
    Monomino,
    Domino,
    LTromino,
    ITromino,
}

impl PieceKindTrait for Mino123 {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            Mino123::Monomino => vec![(0, 0)],
            Mino123::Domino => vec![(0, 0), (0, 1)],
            Mino123::LTromino => vec![(-1, 0), (0, 1), (-1, 1)],
            Mino123::ITromino => vec![(0, -1), (0, 0), (0, 1)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            Mino123::Monomino => (0, CoordsFloat::zero()),
            Mino123::Domino => (
                0,
                match rotation_state {
                    RotationState::Initial => CoordsFloat(0.5, 0.5),
                    RotationState::Cw => CoordsFloat(0.5, -0.5),
                    RotationState::Flipped => CoordsFloat(-0.5, -0.5),
                    RotationState::Ccw => CoordsFloat(-0.5, 0.5),
                },
            ),
            Mino123::LTromino => (2, CoordsFloat::zero()),
            Mino123::ITromino => (1, CoordsFloat::zero()),
        }
    }

    fn detect_spin(&self, _: &DefaultField) -> (Option<PieceKind>, bool) { (None, false) }

    fn asset_name(&self) -> &str {
        match self {
            Mino123::Monomino => "o",
            Mino123::Domino => "s",
            Mino123::LTromino => "l",
            Mino123::ITromino => "i",
        }
    }

    fn iter() -> Box<dyn Iterator<Item = PieceKind>> {
        Box::new(<Self as IntoEnumIterator>::iter().map(|p| PieceKind::Mino123(p)))
    }

    fn n_kinds() -> usize { 4 }
}
