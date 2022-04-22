use crate::{KickTable180, PieceKind, RotationState, Coords, KickTable};

pub mod mino123;
pub mod tetromino;
// pub mod mino1234;

// kicks left, right, or up by one square
pub struct LruKickTable;

impl KickTable for LruKickTable {
    fn rotate_cw(&self, _: PieceKind, _: RotationState) -> Vec<Coords> {
        vec![Coords(0, 0), Coords(0, -1), Coords(0, 1), Coords(-1, 0)]
    }

    fn rotate_ccw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state)
    }
}

impl KickTable180 for LruKickTable {
    fn rotate_180(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords> {
        self.rotate_cw(piece, rotation_state)
    }
}
