use crate::{Bag, Coords, CoordsFloat, KickTable, KickTable180, PieceKind, RotationState};

#[derive(Copy, Clone)]
pub enum Square<P: PieceKind> {
    Empty,
    Filled(P),
}

impl<P: PieceKind> Square<P> {
    fn is_empty(&self) -> bool { matches!(self, Square::Empty) }
}

#[derive(Clone)]
pub struct Line<P: PieceKind> {
    squares: Vec<Square<P>>,
}

impl<P: PieceKind> Line<P> {
    pub fn new(width: usize) -> Self {
        Line {
            squares: (0..width).map(|_| Square::Empty).collect(),
        }
    }

    pub fn squares(&self) -> &[Square<P>] { &self.squares }

    // all squares are filled (not empty or solid garbage)
    fn is_clear(&self) -> bool { self.squares.iter().all(|s| matches!(s, Square::Filled(_))) }

    fn get(&self, i: usize) -> Square<P> { self.squares[i] }

    fn get_mut(&mut self, i: usize) -> &mut Square<P> { &mut self.squares[i] }
}

pub struct LivePiece<P: PieceKind> {
    kind: P,
    coords: Vec<Coords>,
    rotation_state: RotationState,
}

impl<P: PieceKind> LivePiece<P> {
    fn new(kind: P, origin: &Coords) -> Self {
        let coords = kind
            .spawn_offsets()
            .into_iter()
            .map(|offset| *origin + offset)
            .collect();

        LivePiece {
            kind,
            coords,
            rotation_state: RotationState::Initial,
        }
    }

    pub fn coords(&self) -> &Vec<Coords> { &self.coords }

    pub fn kind(&self) -> P { self.kind }

    pub fn rotation_state(&self) -> RotationState { self.rotation_state }

    fn shifted(&self, rows: i32, cols: i32) -> LivePiece<P> {
        let coords = self
            .coords
            .iter()
            .map(|Coords(row, col)| Coords(row + rows, col + cols))
            .collect();

        LivePiece { coords, ..(*self) }
    }

    // these rotations do not use kicks
    fn rotated_cw(&self) -> LivePiece<P> {
        let (pivot_index, offset) = self.kind.pivot_offset(self.rotation_state);
        let pivot = self.coords[pivot_index].to_coords_float() + offset;

        let rotation_state = self.rotation_state.next_cw();
        let coords = self
            .coords
            .iter()
            .map(|c| c.to_coords_float() - pivot)
            .map(|CoordsFloat(row, col)| (CoordsFloat(col, -row) + pivot).to_coords())
            .collect();

        LivePiece {
            coords,
            rotation_state,
            ..(*self)
        }
    }

    fn rotated_ccw(&self) -> LivePiece<P> { self.rotated_cw().rotated_cw().rotated_cw() }

    fn rotated_180(&self) -> LivePiece<P> { self.rotated_cw().rotated_cw() }

    // shadow piece, hard drop position, etc.
    fn projected_down(&self, field: &DefaultField<P>) -> LivePiece<P> {
        let shifted = self.shifted(1, 0);
        if shifted.is_blocked(Some(&self), field) {
            LivePiece {
                coords: self.coords.clone(),
                ..(*self)
            }
        } else {
            shifted.projected_down(field)
        }
    }

    // if the piece being checked has a previous state, `old_piece` should represent that state
    fn is_blocked(&self, old_piece: Option<&LivePiece<P>>, field: &DefaultField<P>) -> bool {
        self.coords
            .iter()
            // make sure the coords are in bounds and are not filled by other pieces
            .any(|c| {
                !field.coords_in_bounds(&c)
                    || !field.get_at(c).is_empty() && old_piece.map(|p| !p.coords.contains(c)).unwrap_or(true)
            })
    }
}

pub struct DefaultField<P: PieceKind> {
    width: usize,
    height: usize,
    hidden: usize,
    queue_len: usize,

    lines: Vec<Line<P>>,

    cur_piece: LivePiece<P>,
    hold_piece: Option<P>,
    hold_swapped: bool,
    piece_origin: Coords,

    lock_delay_actions: Option<usize>,
}

impl<P: PieceKind> DefaultField<P> {
    pub fn new(width: usize, height: usize, hidden: usize, queue_len: usize, bag: &mut impl Bag<P>) -> Self {
        // coordinates of the center (left-aligned) of the bottom-most line of pieces spawned on this field
        // i.e. the coordinates of the @ sign in the following 10-wide field:
        // |    #     |
        // |   #@#    |
        // note how the center is left-aligned for even field widths
        let piece_origin = Coords(hidden as i32 - 2, width as i32 / 2 - 1);

        let mut field = DefaultField {
            width,
            height,
            hidden,
            queue_len,

            lines: (0..height).map(|_| Line::new(width)).collect(),

            cur_piece: LivePiece::new(bag.next(), &piece_origin),
            hold_piece: None,
            hold_swapped: false,
            piece_origin,

            lock_delay_actions: None,
        };
        field.draw_cur_piece();
        field
    }

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn coords_in_bounds(&self, Coords(row, col): &Coords) -> bool {
        (0..self.height as i32).contains(row) && (0..self.width as i32).contains(col)
    }

    pub fn hidden(&self) -> usize { self.hidden }

    pub fn queue_len(&self) -> usize { self.queue_len }

    pub fn lines(&self) -> &[Line<P>] { &self.lines }

    pub fn get_at(&self, Coords(row, col): &Coords) -> Square<P> { self.lines[*row as usize].get(*col as usize) }

    fn set_at(&mut self, Coords(row, col): &Coords, square: Square<P>) {
        *self.lines[*row as usize].get_mut(*col as usize) = square;
    }

    pub fn cur_piece(&self) -> &LivePiece<P> { &self.cur_piece }

    pub fn hold_piece(&self) -> Option<P> { self.hold_piece }

    pub fn shadow_piece(&self) -> LivePiece<P> { self.cur_piece.projected_down(&self) }

    pub fn actions_since_lock_delay(&self) -> Option<usize> { self.lock_delay_actions }

    // used to check whether to activate lock delay
    pub fn cur_piece_cannot_move_down(&self) -> bool {
        self.cur_piece.shifted(1, 0).is_blocked(Some(&self.cur_piece), &self)
    }

    pub fn activate_lock_delay(&mut self) { self.lock_delay_actions.get_or_insert(0); }

    fn update_lock_delay(&mut self, action: bool) -> bool {
        if action {
            if let Some(ref mut n_actions) = self.lock_delay_actions {
                *n_actions += 1;
            }
        }
        action
    }

    // move the current piece to a different position (fails if blocked)
    pub fn try_shift(&mut self, rows: i32, cols: i32) -> bool {
        let action = self.try_update_cur_piece(self.cur_piece.shifted(rows, cols));
        self.update_lock_delay(action)
    }

    pub fn try_rotate_cw(&mut self, kick_table: &impl KickTable<P>) -> bool {
        let kicks = kick_table.rotate_cw(self.cur_piece.kind(), self.cur_piece.rotation_state());
        let rotated = self.cur_piece.rotated_cw();
        let action = self.try_rotate_with_kicks(kicks, rotated);
        self.update_lock_delay(action)
    }

    pub fn try_rotate_ccw(&mut self, kick_table: &impl KickTable<P>) -> bool {
        let kicks = kick_table.rotate_ccw(self.cur_piece.kind(), self.cur_piece.rotation_state());
        let rotated = self.cur_piece.rotated_ccw();
        let action = self.try_rotate_with_kicks(kicks, rotated);
        self.update_lock_delay(action)
    }

    pub fn try_rotate_180(&mut self, kick_table: &impl KickTable180<P>) -> bool {
        let kicks = kick_table.rotate_180(self.cur_piece.kind(), self.cur_piece.rotation_state());
        let rotated = self.cur_piece.rotated_180();
        let action = self.try_rotate_with_kicks(kicks, rotated);
        self.update_lock_delay(action)
    }

    // tries kicks on a rotated piece, swapping with the current piece if one fits
    fn try_rotate_with_kicks(&mut self, kicks: Vec<Coords>, rotated: LivePiece<P>) -> bool {
        kicks
            .into_iter()
            .map(|Coords(rows, cols)| rotated.shifted(rows, cols)) // apply kick to rotated piece
            .find(|piece| !piece.is_blocked(Some(&self.cur_piece), &self)) // first kick that isn't blcoked
            .map(|piece| self.try_update_cur_piece(piece)) // update if a fitting kicked rotation exists
            .unwrap_or(false)
    }

    // tries to spawn a new piece using the provided bag, without erasing the current piece
    // behaves like locking the current piece and spawning a new one
    pub fn try_spawn_no_erase(&mut self, bag: &mut impl Bag<P>) -> bool {
        let kind = bag.next();
        let new_piece = LivePiece::new(kind, &self.piece_origin);

        let blocked = new_piece.is_blocked(None, &self);
        if !blocked {
            self.cur_piece = new_piece;
            self.draw_cur_piece();
        }
        !blocked
    }

    // same as `try_spawn_no_erase` but erases the current piece
    // behaves like swapping out a hold piece
    pub fn try_spawn(&mut self, bag: &mut impl Bag<P>) -> bool {
        let kind = bag.next();
        self.try_update_cur_piece(LivePiece::new(kind, &self.piece_origin))
    }

    pub fn swap_hold_piece(&mut self, bag: &mut impl Bag<P>) -> bool {
        if self.hold_swapped {
            false
        } else {
            self.lock_delay_actions = None;
            self.hold_swapped = true;

            let hold_kind = self.hold_piece;
            self.hold_piece = Some(self.cur_piece.kind());

            if let Some(kind) = hold_kind {
                self.try_update_cur_piece(LivePiece::new(kind, &self.piece_origin))
            } else {
                self.try_spawn(bag)
            }
        }
    }

    // swap the current piece with the shadow piece
    pub fn project_down(&mut self) -> bool {
        let projected = self.cur_piece.projected_down(&self);
        self.try_update_cur_piece(projected)
    }

    pub fn hard_drop(&mut self, bag: &mut impl Bag<P>) -> bool {
        self.lock_delay_actions = None;
        self.hold_swapped = false;

        self.project_down();
        self.clear_lines();
        self.try_spawn_no_erase(bag)
    }

    pub fn clear_lines(&mut self) {
        let uncleared_lines = self
            .lines
            .iter()
            .filter(|l| !l.is_clear())
            .map(|l| l.clone())
            .collect::<Vec<_>>();

        let n_cleared = self.height - uncleared_lines.len();
        self.lines = (0..n_cleared).map(|_| Line::new(self.width)).collect();
        self.lines.extend(uncleared_lines);
    }

    // changes and redraws the current piece if the new piece isn't blocked
    fn try_update_cur_piece(&mut self, new_piece: LivePiece<P>) -> bool {
        let blocked = new_piece.is_blocked(Some(&self.cur_piece), &self);
        if !blocked {
            self.erase_cur_piece();
            self.draw_piece(&new_piece);
            self.cur_piece = new_piece;
        }
        !blocked
    }

    fn erase_cur_piece(&mut self) {
        for coords in self.cur_piece.coords.clone() {
            self.set_at(&coords, Square::Empty);
        }
    }

    fn draw_piece(&mut self, piece: &LivePiece<P>) {
        for coords in piece.coords() {
            self.set_at(coords, Square::Filled(piece.kind()));
        }
    }

    fn draw_cur_piece(&mut self) {
        for coords in self.cur_piece.coords().clone() {
            self.set_at(&coords, Square::Filled(self.cur_piece.kind()));
        }
    }
}
