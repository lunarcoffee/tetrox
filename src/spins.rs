use crate::{
    field::DefaultField,
    pieces::{
        mino1234::Mino1234,
        tetromino::{TetrominoAsc, TetrominoSrs},
        PieceKind,
    },
    Coords,
};

use num_traits::ToPrimitive;

pub trait SpinDetector {
    // returns the type of spin after a hard drop (if any) and whether it is mini
    // `field.cur_piece()` is the piece that was just dropped
    fn detect(&self, field: &DefaultField) -> (Option<PieceKind>, bool);
}

pub struct TSpinDetector;

impl TSpinDetector {
    fn is_t(kind: PieceKind) -> bool {
        matches!(
            kind,
            PieceKind::TetrominoSrs(TetrominoSrs::T)
                | PieceKind::TetrominoAsc(TetrominoAsc::T)
                | PieceKind::Mino1234(Mino1234::TetrominoSrs(TetrominoSrs::T))
        )
    }
}

impl SpinDetector for TSpinDetector {
    fn detect(&self, field: &DefaultField) -> (Option<PieceKind>, bool) {
        let piece = field.cur_piece();
        let kind = piece.kind();
        if !Self::is_t(kind) || !field.last_move_rotated() {
            return (None, false);
        }

        let center = piece.coords()[1];
        let mut corner_offsets = [(-1, -1), (-1, 1), (1, 1), (1, -1)];
        corner_offsets.rotate_left(piece.rotation_state().to_usize().unwrap());

        let offset_filled = corner_offsets
            .into_iter()
            .map(|o| center + Coords(o.0, o.1))
            .map(|c| field.get_at(&c).map(|s| s.is_filled() as usize).unwrap_or(1))
            .collect::<Vec<_>>();

        let n_filled_front = offset_filled[0] + offset_filled[1];
        let n_filled_back = offset_filled[2] + offset_filled[3];

        // two filled front corners and one or more filled back corners is a t-spin
        if n_filled_front == 2 && n_filled_back > 0 {
            (Some(kind), false)
        } else if n_filled_front == 1 && n_filled_back == 2 {
            // one filled front corner and two filled back corners is a t-spin mini, unless the last kick on
            // the piece kicked it one column and two rows; then it is a regular t-spin
            let last_was_1_2_kick = field.last_cur_piece_kick().map(|c| c.0.abs() == 2 && c.1.abs() == 1);
            (Some(kind), !last_was_1_2_kick.unwrap_or(false))
        } else {
            (None, false)
        }
    }
}

pub struct ImmobileSpinDetector;

impl SpinDetector for ImmobileSpinDetector {
    fn detect(&self, field: &DefaultField) -> (Option<PieceKind>, bool) {
        let piece = field.cur_piece();
        let is_immobile = [(0, -1), (0, 1), (-1, 0)]
            .into_iter()
            .all(|o| piece.shifted(o.0, o.1).is_blocked(Some(piece), field));
        let is_spin = is_immobile && field.last_move_rotated();
        
        (is_spin.then(|| piece.kind()), false)
    }
}

pub struct NoSpinDetector;

impl SpinDetector for NoSpinDetector {
    fn detect(&self, _: &DefaultField) -> (Option<PieceKind>, bool) { (None, false) }
}
