use crate::{PieceKind, Coords, RotationState, Bag};

#[derive(Copy, Clone)]
pub enum Square<P: PieceKind> {
    Empty,
    Filled(P),
}

impl<P: PieceKind> Square<P> {
    fn is_empty(&self) -> bool { matches!(self, Square::Empty) }
}

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

    fn is_clear(&self) -> bool { self.squares.iter().all(|s| matches!(s, Square::Empty)) }

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

    fn shifted(&self, rows: i32, cols: i32) -> LivePiece<P> {
        let coords = self
            .coords
            .iter()
            .map(|Coords(row, col)| Coords(row + rows, col + cols))
            .collect();
        LivePiece { coords, ..(*self) }
    }

    fn rotated_cw(&self) -> LivePiece<P> { // TODO: need pivot coords
        let coords = self.coords.iter().map(|Coords(row, col)| Coords(*col, -row)).collect();
        LivePiece { coords, ..(*self) }
    }

    fn rotate_ccw(&self) -> LivePiece<P> {
        let coords = self.coords.iter().map(|Coords(row, col)| Coords(-col, *row)).collect();
        LivePiece { coords, ..(*self) }
    }

    fn rotated_180(&self) -> LivePiece<P> { self.rotated_cw().rotated_cw() }

    fn is_blocked(&self, old_piece: &LivePiece<P>, field: &DefaultField<P>) -> bool {
        self.coords
            .iter()
            // make sure the coords are in bounds and are not filled by other pieces
            .any(|c| !field.coords_in_bounds(&c) || !field.get_at(c).is_empty() && !old_piece.coords.contains(c))
    }
}

pub struct DefaultField<P: PieceKind> {
    width: usize,
    height: usize,
    hidden: usize,

    lines: Vec<Line<P>>,

    cur_piece: LivePiece<P>,
    held_piece: Option<LivePiece<P>>,
    piece_origin: Coords,
}

impl<P: PieceKind> DefaultField<P> {
    pub fn new(width: usize, height: usize, hidden: usize, bag: &mut impl Bag<P>) -> Self {
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
            lines: (0..height).map(|_| Line::new(width)).collect(),
            cur_piece: LivePiece::new(bag.next(), &piece_origin),
            held_piece: None,
            piece_origin,
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

    pub fn lines(&self) -> &[Line<P>] { &self.lines }

    pub fn get_at(&self, Coords(row, col): &Coords) -> Square<P> { self.lines[*row as usize].get(*col as usize) }

    fn set_at(&mut self, Coords(row, col): &Coords, square: Square<P>) {
        *self.lines[*row as usize].get_mut(*col as usize) = square;
    }

    pub fn cur_piece(&self) -> &LivePiece<P> { &self.cur_piece }

    pub fn held_piece(&self) -> Option<&LivePiece<P>> { self.held_piece.as_ref() }

    // move the current piece to a different position (fails if blocked)
    pub fn try_shift(&mut self, rows: i32, cols: i32) -> bool {
        let updated = self.cur_piece.shifted(rows, cols);
        self.try_update_cur_piece(updated)
    }

    // tries to spawn a new piece using the provided bag, without erasing the current piece
    // behaves like locking the current piece and spawning a new one
    pub fn try_spawn_no_erase(&mut self, bag: &mut impl Bag<P>) -> bool {
        let kind = bag.next();
        let new_piece = LivePiece::new(kind, &self.piece_origin);

        let blocked = new_piece.is_blocked(&self.cur_piece, &self);
        if !blocked {
            self.cur_piece = new_piece;
            self.draw_cur_piece();
        }
        !blocked
    }

    // same as `try_spawn_no_erase` but erases the current piece
    // behaves like swapping out a held piece
    pub fn try_spawn(&mut self, bag: &mut impl Bag<P>) -> bool {
        let kind = bag.next();
        self.try_update_cur_piece(LivePiece::new(kind, &self.piece_origin))
    }

    // changes and redraws the current piece if the new piece isn't blocked
    fn try_update_cur_piece(&mut self, new_piece: LivePiece<P>) -> bool {
        let blocked = new_piece.is_blocked(&self.cur_piece, &self);
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
