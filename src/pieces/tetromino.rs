use num_traits::ToPrimitive;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{field::DefaultField, Coords, CoordsFloat, PieceKind, kicks::RotationState};

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

    fn detect_spin(&self, field: &DefaultField) -> (Option<PieceKind>, bool) {
        let piece = field.cur_piece();
        if let kind @ PieceKind::TetrominoSrs(TetrominoSrs::T) = piece.kind() {
            if field.last_move_rotated() {
                let center = piece.coords()[1];
                let mut corner_offsets = vec![(-1, -1), (-1, 1), (1, 1), (1, -1)];
                corner_offsets.rotate_left(piece.rotation_state().to_usize().unwrap());

                let offset_filled = corner_offsets
                    .into_iter()
                    .map(|(row, col)| {
                        field
                            .get_at(&(center + Coords(row, col))) // get corner at given offset
                            .map(|s| s.is_filled() as usize) // 1 if filled, 0 if empty
                            .unwrap_or(1) // consider out of bounds areas filled (e.g. field walls)
                    })
                    .collect::<Vec<_>>();

                let n_filled_front = offset_filled[0] + offset_filled[1];
                let n_filled_back = offset_filled[2] + offset_filled[3];

                // two filled front corners and one or more filled back corners is a t-spin
                if n_filled_front == 2 && n_filled_back > 0 {
                    return (Some(kind), false);
                } else if n_filled_front == 1 && n_filled_back == 2 {
                    // one filled front corner and two filled back corners is a t-spin mini, unless the last kick on
                    // the piece kicked it one column and two rows; then it is a regular t-spin
                    let last_was_1_2_kick = field
                        .last_cur_piece_kick()
                        .map(|Coords(row, col)| row.abs() == 2 && col.abs() == 1)
                        .unwrap_or(false);
                    return (Some(kind), !last_was_1_2_kick);
                }
            }
        }
        (None, false)
    }

    fn asset_name(&self) -> &str {
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

    // TODO: make this work lol
    fn detect_spin(&self, field: &DefaultField) -> (Option<PieceKind>, bool) { self.to_srs().detect_spin(field) }

    fn asset_name(&self) -> &str {
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

    fn iter() -> Box<dyn Iterator<Item = PieceKind>> {
        Box::new(<Self as IntoEnumIterator>::iter().map(|p| PieceKind::TetrominoAsc(p)))
    }

    fn n_kinds() -> usize { 7 }
}
