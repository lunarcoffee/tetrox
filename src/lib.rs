use std::{iter, ops};

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Coords(i32, i32);

impl ops::Add for Coords {
    type Output = Coords;

    fn add(self, Coords(row2, col2): Self) -> Self::Output { Coords(self.0 + row2, self.1 + col2) }
}

impl ops::Sub for Coords {
    type Output = Coords;

    fn sub(self, Coords(row2, col2): Self) -> Self::Output { Coords(self.0 - row2, self.1 - col2) }
}

impl ops::Neg for Coords {
    type Output = Coords;

    fn neg(self) -> Self::Output { Coords(-self.0, -self.1) }
}

pub trait PieceKind: Copy {
    fn spawn_offsets(&self) -> Vec<Coords>;
}

#[derive(Copy, Clone, FromPrimitive)]
pub enum Tetromino {
    S,
    Z,
    L,
    J,
    I,
    O,
    T,
}

impl PieceKind for Tetromino {
    fn spawn_offsets(&self) -> Vec<Coords> {
        match self {
            Tetromino::S => [(0, -1), (0, 0), (-1, 0), (-1, 1)],
            Tetromino::Z => [(0, 0), (0, 1), (-1, -1), (-1, 0)],
            Tetromino::L => [(0, -1), (0, 0), (0, 1), (-1, 1)],
            Tetromino::J => [(0, -1), (0, 0), (0, 1), (-1, -1)],
            Tetromino::I => [(0, -1), (0, 0), (0, 1), (0, 2)],
            Tetromino::O => [(0, 0), (0, 1), (-1, 0), (-1, 1)],
            Tetromino::T => [(0, -1), (0, 0), (0, 1), (-1, 0)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }
}

pub trait Bag<P: PieceKind> {
    fn next(&mut self) -> P;
    fn peek(&mut self) -> Box<dyn Iterator<Item = &P> + '_>;

    fn lookahead(&self) -> usize;
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

            let mut rng = rand::thread_rng();
            self.next_bag = (0..7)
                .map(|_| Tetromino::from_i32(rng.gen_range(0..7)).unwrap())
                .collect();
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
        Box::new(self.cur_bag.iter().chain(self.next_bag.iter()).take(7))
    }

    fn lookahead(&self) -> usize { 7 }
}

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum RotationState {
    Initial,
    Cw,
    Flipped,
    Ccw,
}

impl RotationState {
    fn next_cw(&self) -> RotationState { RotationState::from_i32((self.to_i32().unwrap() + 1) % 4).unwrap() }

    fn next_ccw(&self) -> RotationState { RotationState::from_i32((self.to_i32().unwrap() + 3) % 4).unwrap() }
}

pub trait KickTable<P: PieceKind> {
    fn rotate_cw(&self, piece: P, rotation_state: RotationState) -> Vec<Coords>;
    fn rotate_ccw(&self, piece: P, rotation_state: RotationState) -> Vec<Coords>;
}

pub trait KickTable180<P: PieceKind>: KickTable<P> {
    fn rotate_180(&self, piece: P, rotation_state: RotationState) -> Vec<Coords>;
}

pub struct SrsKickTable;

impl KickTable<Tetromino> for SrsKickTable {
    fn rotate_cw(&self, piece: Tetromino, rotation_state: RotationState) -> Vec<Coords> {
        match piece {
            Tetromino::O => vec![(0, 0)],
            Tetromino::I => match rotation_state {
                RotationState::Initial => vec![(0, 0), (0, -2), (0, 1), (-1, -2), (2, 1)],
                RotationState::Cw => vec![(0, 0), (0, -1), (0, 2), (2, -1), (-1, 2)],
                RotationState::Flipped => vec![(0, 0), (0, 2), (0, -1), (1, 2), (-2, -1)],
                RotationState::Ccw => vec![(0, 0), (0, 1), (0, -2), (-2, 1), (1, -2)],
            },
            _ => match rotation_state {
                RotationState::Initial => vec![(0, 0), (0, -1), (1, -1), (-2, 0), (-2, -1)],
                RotationState::Cw => vec![(0, 0), (0, 1), (-1, 1), (2, 0), (2, 1)],
                RotationState::Flipped => vec![(0, 0), (0, 1), (1, 1), (-2, 0), (-2, 1)],
                RotationState::Ccw => vec![(0, 0), (0, -1), (-1, -1), (2, 0), (2, -1)],
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
            RotationState::Initial => vec![(0, 0), (0, 1), (1, 1), (-1, 1), (1, 0), (-1, 0)],
            RotationState::Cw => vec![(0, 0), (1, 0), (1, 2), (1, 1), (0, 2), (0, 1)],
            RotationState::Flipped => vec![(0, 0), (0, -1), (-1, -1), (1, -1), (-1, 0), (1, 0)],
            RotationState::Ccw => vec![(0, 0), (-1, 0), (-1, 2), (-1, 1), (0, 2), (0, 1)],
        }
        .into_iter()
        .map(|(row_shift, col_shift)| Coords(row_shift, col_shift))
        .collect::<Vec<_>>()
    }
}

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

    fn rotated_cw(&self) -> LivePiece<P> {
        let coords = self.coords.iter().map(|Coords(row, col)| Coords(*col, -row)).collect();
        LivePiece { coords, ..(*self) }
    }

    fn rotate_ccw(&self) -> LivePiece<P> {
        let coords = self.coords.iter().map(|Coords(row, col)| Coords(-col, *row)).collect();
        LivePiece { coords, ..(*self) }
    }

    fn rotated_180(&self) -> LivePiece<P> { self.rotated_cw().rotated_cw() }

    fn is_blocked(&self, field: &DefaultField<P>) -> bool {
        // TODO: check bounds
        self.coords.iter().any(|c| !field.get_at(c).is_empty())
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

        DefaultField {
            width,
            height,
            hidden,
            lines: (0..height).map(|_| Line::new(width)).collect(),
            cur_piece: LivePiece::new(bag.next(), &piece_origin),
            held_piece: None,
            piece_origin,
        }
    }

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

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
        self.cur_piece = LivePiece::new(kind, &self.piece_origin);

        let blocked = self.cur_piece.is_blocked(&self);
        if blocked {
            self.draw_cur_piece();
        }
        blocked
    }

    // same as `try_spawn_no_erase` but erases the current piece
    // behaves like swapping out a held piece
    pub fn try_spawn(&mut self, bag: &mut impl Bag<P>) -> bool {
        let kind = bag.next();
        self.try_update_cur_piece(LivePiece::new(kind, &self.piece_origin))
    }

    fn try_update_cur_piece(&mut self, new_piece: LivePiece<P>) -> bool {
        let blocked = new_piece.is_blocked(&self);
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
