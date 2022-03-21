use std::ops;

use rand::prelude::SliceRandom;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{Bag, Coords, CoordsFloat, KickTable, KickTable180, PieceKind, RotationState};

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, Eq, Hash)]
pub enum Tetromino {
    S,
    Z,
    L,
    J,
    T,
    O,
    I,
}

impl PieceKind for Tetromino {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            Tetromino::S => [(0, -1), (0, 0), (-1, 0), (-1, 1)],
            Tetromino::Z => [(0, 0), (0, 1), (-1, -1), (-1, 0)],
            Tetromino::L => [(0, -1), (0, 0), (0, 1), (-1, 1)],
            Tetromino::J => [(0, -1), (0, 0), (0, 1), (-1, -1)],
            Tetromino::T => [(0, -1), (0, 0), (0, 1), (-1, 0)],
            Tetromino::O => [(0, 0), (0, 1), (-1, 0), (-1, 1)],
            Tetromino::I => [(0, -1), (0, 0), (0, 1), (0, 2)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }

    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        match self {
            Tetromino::S => (1, CoordsFloat::zero()),
            Tetromino::Z => (0, CoordsFloat::zero()),
            Tetromino::L => (1, CoordsFloat::zero()),
            Tetromino::J => (1, CoordsFloat::zero()),
            Tetromino::T => (1, CoordsFloat::zero()),
            Tetromino::O => (2, Tetromino::I.pivot_offset(rotation_state).1),
            Tetromino::I => (
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

    fn asset_name(&self) -> &str {
        match self {
            Tetromino::S => "s",
            Tetromino::Z => "z",
            Tetromino::L => "l",
            Tetromino::J => "j",
            Tetromino::T => "t",
            Tetromino::O => "o",
            Tetromino::I => "i",
        }
    }
}

pub struct SevenBag {
    cur_bag: Vec<Tetromino>,
    next_bag: Vec<Tetromino>,
}

impl SevenBag {
    pub fn new() -> Self {
        let mut bag = SevenBag {
            cur_bag: vec![],
            next_bag: vec![],
        };
        bag.update_bags();
        bag
    }

    fn update_bags(&mut self) {
        if self.cur_bag.is_empty() {
            self.cur_bag.extend(&self.next_bag);

            self.next_bag = Tetromino::iter().collect::<Vec<_>>();
            self.next_bag.shuffle(&mut rand::thread_rng());
        }
    }
}

impl Bag<Tetromino> for SevenBag {
    fn next(&mut self) -> Tetromino {
        self.update_bags();
        self.cur_bag.pop().unwrap()
    }

    fn peek(&mut self) -> Box<dyn Iterator<Item = &Tetromino> + '_> {
        self.update_bags();
        Box::new(self.cur_bag.iter().rev().chain(self.next_bag.iter().rev()).take(7))
    }

    fn lookahead(&self) -> usize { 7 }
}

pub struct SrsKickTable;

impl KickTable<Tetromino> for SrsKickTable {
    fn rotate_cw(&self, piece: Tetromino, rotation_state: RotationState) -> Vec<Coords> {
        match piece {
            Tetromino::O => vec![(0, 0)],
            Tetromino::I => match rotation_state {
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

    fn rotate_ccw(&self, piece: Tetromino, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state.next_ccw())
            .into_iter()
            .map(ops::Neg::neg)
            .collect()
    }
}

// has non-guideline srs 180 kicks
pub struct ExtendedSrsKickTable;

impl KickTable<Tetromino> for ExtendedSrsKickTable {
    fn rotate_cw(&self, piece: Tetromino, rotation_state: RotationState) -> Vec<Coords> {
        SrsKickTable.rotate_cw(piece, rotation_state)
    }

    fn rotate_ccw(&self, piece: Tetromino, rotation_state: RotationState) -> Vec<Coords> {
        SrsKickTable.rotate_ccw(piece, rotation_state)
    }
}

impl KickTable180<Tetromino> for ExtendedSrsKickTable {
    fn rotate_180(&self, _piece: Tetromino, rotation_state: RotationState) -> Vec<Coords> {
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
