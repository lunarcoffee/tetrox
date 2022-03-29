use std::{mem, ops};

use num_traits::ToPrimitive;
use rand::prelude::SliceRandom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{field::DefaultField, Bag, Coords, CoordsFloat, KickTable, KickTable180, PieceKind, RotationState};

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum SrsTetromino {
    S,
    Z,
    L,
    J,
    T,
    O,
    I,
}

impl PieceKind for SrsTetromino {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            SrsTetromino::S => [(0, -1), (0, 0), (-1, 0), (-1, 1)],
            SrsTetromino::Z => [(0, 0), (0, 1), (-1, -1), (-1, 0)],
            SrsTetromino::L => [(0, -1), (0, 0), (0, 1), (-1, 1)],
            SrsTetromino::J => [(0, -1), (0, 0), (0, 1), (-1, -1)],
            SrsTetromino::T => [(0, -1), (0, 0), (0, 1), (-1, 0)],
            SrsTetromino::O => [(0, 0), (0, 1), (-1, 0), (-1, 1)],
            SrsTetromino::I => [(0, -1), (0, 0), (0, 1), (0, 2)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            SrsTetromino::S => (1, CoordsFloat::zero()),
            SrsTetromino::Z => (0, CoordsFloat::zero()),
            SrsTetromino::L => (1, CoordsFloat::zero()),
            SrsTetromino::J => (1, CoordsFloat::zero()),
            SrsTetromino::T => (1, CoordsFloat::zero()),
            SrsTetromino::O => (2, SrsTetromino::I.pivot_offset(rotation_state).1),
            SrsTetromino::I => (
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

    fn detect_spin(&self, field: &DefaultField<Self>) -> (Option<Self>, bool) {
        let piece = field.cur_piece();
        if let kind @ SrsTetromino::T = piece.kind() {
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
            SrsTetromino::S => "s",
            SrsTetromino::Z => "z",
            SrsTetromino::L => "l",
            SrsTetromino::J => "j",
            SrsTetromino::T => "t",
            SrsTetromino::O => "o",
            SrsTetromino::I => "i",
        }
    }

    fn iter() -> Box<dyn Iterator<Item = Self>> { Box::new(<Self as IntoEnumIterator>::iter()) }

    fn n_kinds() -> usize { 7 }
}

pub struct SingleBag<P: PieceKind> {
    bag: Vec<P>,
}

impl<P: PieceKind> SingleBag<P> {
    pub fn new() -> Self {
        let mut bag = SingleBag { bag: vec![] };
        bag.update_bag();
        bag.update_bag();
        bag
    }

    fn update_bag(&mut self) {
        if self.bag.len() <= P::n_kinds() {
            let mut next_bag = P::iter().collect::<Vec<_>>();
            next_bag.shuffle(&mut rand::thread_rng());

            // prepend to preserve peek order
            mem::swap(&mut self.bag, &mut next_bag);
            self.bag.extend(next_bag);
        }
    }
}

impl<P: PieceKind> Bag<P> for SingleBag<P> {
    fn next(&mut self) -> P {
        self.update_bag();
        self.bag.pop().unwrap()
    }

    fn peek(&mut self) -> Box<dyn Iterator<Item = &P> + '_> {
        self.update_bag();
        Box::new(self.bag.iter().rev())
    }

    fn lookahead(&self) -> usize { P::n_kinds() }
}

pub struct SrsKickTable;

impl KickTable<SrsTetromino> for SrsKickTable {
    fn rotate_cw(&self, piece: SrsTetromino, rotation_state: RotationState) -> Vec<Coords> {
        match piece {
            SrsTetromino::O => vec![(0, 0)],
            SrsTetromino::I => match rotation_state {
                RotationState::Initial => vec![(0, 0), (0, -2), (0, 1), (1, -2), (-2, 1)],
                RotationState::Cw => vec![(0, 0), (0, -1), (0, 2), (-2, -1), (1, 2)],
                RotationState::Flipped => vec![(0, 0), (0, 2), (0, -1), (-1, 2), (2, -1)],
                RotationState::Ccw => vec![(0, 0), (0, 1), (0, -2), (2, 1), (-1, -2)],
            },
            _ => match rotation_state {
                RotationState::Initial => vec![(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
                RotationState::Cw => vec![(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                RotationState::Flipped => vec![(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                RotationState::Ccw => vec![(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
            },
        }
        .into_iter()
        .map(|(row_shift, col_shift)| Coords(row_shift, col_shift))
        .collect::<Vec<_>>()
    }

    fn rotate_ccw(&self, piece: SrsTetromino, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state.next_ccw())
            .into_iter()
            .map(ops::Neg::neg)
            .collect()
    }
}

// 180 rotate kick table from tetr.io
pub struct Tetrio180KickTable;

impl KickTable<SrsTetromino> for Tetrio180KickTable {
    fn rotate_cw(&self, piece: SrsTetromino, rotation_state: RotationState) -> Vec<Coords> {
        SrsKickTable.rotate_cw(piece, rotation_state)
    }

    fn rotate_ccw(&self, piece: SrsTetromino, rotation_state: RotationState) -> Vec<Coords> {
        SrsKickTable.rotate_ccw(piece, rotation_state)
    }
}

impl KickTable180<SrsTetromino> for Tetrio180KickTable {
    fn rotate_180(&self, _piece: SrsTetromino, rotation_state: RotationState) -> Vec<Coords> {
        match rotation_state {
            RotationState::Initial => vec![(0, 0), (-1, 0), (-1, 1), (-1, -1), (0, 1), (0, -1)],
            RotationState::Cw => vec![(0, 0), (0, 1), (-2, 1), (-1, 1), (-2, 0), (-1, 0)],
            RotationState::Flipped => vec![(0, 0), (1, 0), (1, -1), (1, 1), (0, -1), (0, 1)],
            RotationState::Ccw => vec![(0, 0), (0, -1), (-2, -1), (-1, -1), (-2, 0), (-1, 0)],
        }
        .into_iter()
        .map(|(row_shift, col_shift)| Coords(row_shift, col_shift))
        .collect::<Vec<_>>()
    }
}
