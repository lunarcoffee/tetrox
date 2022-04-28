use std::ops;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::{
    pieces::{tetromino::TetrominoSrs, PieceKind},
    Coords,
};

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum RotationState {
    Initial,
    Cw,
    Flipped,
    Ccw,
}

impl RotationState {
    pub fn next_cw(&self) -> RotationState { RotationState::from_i32((self.to_i32().unwrap() + 1) % 4).unwrap() }

    pub fn next_ccw(&self) -> RotationState { RotationState::from_i32((self.to_i32().unwrap() + 3) % 4).unwrap() }
}

// cw/ccw kick table
pub trait KickTable {
    fn rotate_cw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords>;

    fn rotate_ccw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords>;
}

pub trait KickTable180 {
    fn rotate_180(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords>;
}

// kicks left, right, or up by one square
pub struct BasicKickTable;

impl KickTable for BasicKickTable {
    fn rotate_cw(&self, _: PieceKind, _: RotationState) -> Vec<Coords> {
        vec![Coords(0, 0), Coords(0, -1), Coords(0, 1), Coords(-1, 0)]
    }

    fn rotate_ccw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state)
    }
}

impl KickTable180 for BasicKickTable {
    fn rotate_180(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state)
    }
}

// standard asymmetrical srs kick table
pub struct SrsKickTable;

impl KickTable for SrsKickTable {
    fn rotate_cw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        match piece {
            PieceKind::TetrominoSrs(kind) => match kind {
                TetrominoSrs::I => match rotation_state {
                    RotationState::Initial => vec![(0, 0), (0, -2), (0, 1), (1, -2), (-2, 1)],
                    RotationState::Cw => vec![(0, 0), (0, -1), (0, 2), (-2, -1), (1, 2)],
                    RotationState::Flipped => vec![(0, 0), (0, 2), (0, -1), (-1, 2), (2, -1)],
                    RotationState::Ccw => vec![(0, 0), (0, 1), (0, -2), (2, 1), (-1, -2)],
                },
                TetrominoSrs::O => vec![], // don't let o rotate at all
                _ => match rotation_state {
                    RotationState::Initial => vec![(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
                    RotationState::Cw => vec![(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                    RotationState::Flipped => vec![(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                    RotationState::Ccw => vec![(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
                },
            },
            _ => vec![(0, 0)],
        }
        .into_iter()
        .map(|(row_shift, col_shift)| Coords(row_shift, col_shift))
        .collect::<Vec<_>>()
    }

    fn rotate_ccw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state.next_ccw())
            .into_iter()
            .map(ops::Neg::neg)
            .collect()
    }
}

// 180 rotate kick table from tetr.io
pub struct TetrIo180KickTable;

impl KickTable180 for TetrIo180KickTable {
    fn rotate_180(&self, _: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
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

// ascension kick table
pub struct AscKickTable;

impl KickTable for AscKickTable {
    fn rotate_cw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_ccw(piece, rotation_state)
            .into_iter()
            .map(|k| Coords(k.0, -k.1))
            .collect()
    }

    fn rotate_ccw(&self, _: PieceKind, _: RotationState) -> Vec<Coords> {
        let right = [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
        let left = [(0, -1), (1, -1), (-1, 0), (-1, 1), (-1, 2), (2, -1), (0, -2), (-2, 0)];
        let other = [(-2, 1), (-2, 2), (1, -2), (2, -2), (-1, -1)];
        let kicks = right.into_iter().chain(left).chain(other);
        kicks
            .map(|(row_shift, col_shift)| Coords(row_shift, col_shift))
            .collect()
    }
}
