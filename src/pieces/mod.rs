use crate::{Coords, KickTable, KickTable180, PieceKind, RotationState};

pub mod mino123;
pub mod tetromino;
pub mod mino1234;
// pub mod mino1234;

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
