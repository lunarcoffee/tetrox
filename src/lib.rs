use std::ops;

use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use rand::Rng;

#[derive(Copy, Clone)]
pub struct Coords(i32, i32);

impl ops::Add for Coords {
    type Output = Coords;

    fn add(self, Coords(row2, col2): Self) -> Self::Output {
        Coords(self.0 + row2, self.1 + col2)
    }
}

impl ops::Sub for Coords {
    type Output = Coords;

    fn sub(self, Coords(row2, col2): Self) -> Self::Output {
        Coords(self.0 - row2, self.1 - col2)
    }
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
            Tetromino::S => [(0, -2), (0, -1), (-1, -1), (-1, 0)],
            Tetromino::Z => [(0, -1), (0, 0), (-1, -2), (-1, -1)],
            Tetromino::L => [(0, -2), (0, -1), (0, 0), (-1, 0)],
            Tetromino::J => [(0, -2), (0, -1), (0, 0), (-1, -2)],
            Tetromino::I => [(0, -2), (0, -1), (0, 0), (0, 1)],
            Tetromino::O => [(0, -1), (0, 0), (-1, -1), (-1, 0)],
            Tetromino::T => [(0, -2), (0, -1), (0, 0), (-1, -1)],
        }
        .into_iter()
        .map(|(row, col)| Coords(row, col))
        .collect()
    }
}

pub trait Bag<P: PieceKind> {
    fn next(&mut self, field: &impl Field<P>) -> P;
    fn peek(&mut self, field: &impl Field<P>) -> Box<dyn Iterator<Item = &P> + '_>;

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
                .map(|_| Tetromino::from_i32(rng.gen_range(0i32..7)).unwrap())
                .collect();
        }
    }
}

impl Bag<Tetromino> for SevenBag {
    fn next(&mut self, _: &impl Field<Tetromino>) -> Tetromino {
        self.update_bags();
        self.cur_bag.pop().unwrap()
    }

    fn peek(&mut self, _: &impl Field<Tetromino>) -> Box<dyn Iterator<Item = &Tetromino> + '_> {
        self.update_bags();
        Box::new(self.cur_bag.iter().chain(self.next_bag.iter()).take(7))
    }

    fn lookahead(&self) -> usize {
        7
    }
}

#[derive(Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum RotationState {
    Initial,
    Cw,
    Flipped,
    Ccw,
}

impl RotationState {
    fn next_cw(&self) -> RotationState {
        RotationState::from_i32((self.to_i32().unwrap() + 1) % 4).unwrap()
    }

    fn next_ccw(&self) -> RotationState {
        RotationState::from_i32((self.to_i32().unwrap() + 3) % 4).unwrap()
    }
}

pub trait KickTable<P: PieceKind> {
    fn rotate_cw(&self, piece: P, rotation_state: RotationState) -> &[Coords];
    fn rotate_ccw(&self, piece: P, rotation_state: RotationState) -> &[Coords];
}

pub trait KickTable180<P: PieceKind>: KickTable<P> {
    fn rotate_180(&self, piece: P, rotation_state: RotationState) -> &[Coords];
}

pub struct SrsKickTable;

impl KickTable<Tetromino> for SrsKickTable {
    fn rotate_cw(&self, piece: Tetromino, rotation_state: RotationState) -> &[Coords] {
        todo!() // TODO
    }
    
    fn rotate_ccw(&self, piece: Tetromino, rotation_state: RotationState) -> &[Coords] {
        todo!() // TODO
    }
}

// has custom 180 kicks
pub struct ExtendedSrsKickTable;

impl KickTable<Tetromino> for ExtendedSrsKickTable {
    fn rotate_cw(&self, piece: Tetromino, rotation_state: RotationState) -> &[Coords] {
        SrsKickTable.rotate_cw(piece, rotation_state)
    }
    
    fn rotate_ccw(&self, piece: Tetromino, rotation_state: RotationState) -> &[Coords] {
        SrsKickTable.rotate_ccw(piece, rotation_state)
    }
}

impl KickTable180<Tetromino> for ExtendedSrsKickTable {
    fn rotate_180(&self, piece: Tetromino, rotation_state: RotationState) -> &[Coords] {
        todo!() // TODO
    }
}

pub struct LivePiece<P: PieceKind> {
    kind: P,
    coords: Vec<Coords>,
    rotation_state: RotationState,
}

impl<P: PieceKind> LivePiece<P> {
    pub fn new(kind: P, origin: Coords) -> Self {
        let coords = kind
            .spawn_offsets()
            .into_iter()
            .map(|offset| origin + offset)
            .collect();

        LivePiece {
            kind,
            coords,
            rotation_state: RotationState::Initial,
        }
    }

    pub fn coords(&self) -> &Vec<Coords> {
        &self.coords
    }

    pub fn kind(&self) -> P {
        self.kind
    }

    pub fn try_rotate_cw(&self, field: &impl Field<P>, kick_table: &impl KickTable<P>) -> bool {
        false
    }

    pub fn try_rotate_ccw(&self, field: &impl Field<P>, kick_table: &impl KickTable<P>) -> bool {
        false
    }

    pub fn try_rotate_180(&self, field: &impl Field<P>, kick_table: &impl KickTable180<P>) -> bool {
        false
    }
}

#[derive(Copy, Clone)]
pub enum Square<P: PieceKind> {
    Empty,
    Filled(P),
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

    fn is_clear(&self) -> bool {
        self.squares.iter().all(|s| matches!(s, Square::Empty))
    }

    fn get(&self, i: usize) -> Square<P> {
        self.squares[i]
    }

    fn get_mut(&mut self, i: usize) -> &mut Square<P> {
        &mut self.squares[i]
    }
}

pub trait Field<P: PieceKind> {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn hidden(&self) -> usize;

    fn lines(&self) -> &[Line<P>];
    fn get_at(&self, coords: Coords) -> Square<P>;

    fn cur_piece(&self) -> Option<&LivePiece<P>>;
    fn held_piece(&self) -> Option<&LivePiece<P>>;

    fn spawn_piece(&mut self, bg: &mut impl Bag<P>);
}

pub struct DefaultField<P: PieceKind> {
    width: usize,
    height: usize,
    hidden: usize,

    lines: Vec<Line<P>>,

    cur_piece: Option<LivePiece<P>>,
    held_piece: Option<LivePiece<P>>,
}

impl<P: PieceKind> DefaultField<P> {
    pub fn new(width: usize, height: usize, hidden: usize) -> Self {
        // TODO: invalid params?
        DefaultField {
            width,
            height,
            hidden,
            lines: (0..height).map(|_| Line::new(width)).collect(),
            cur_piece: None,
            held_piece: None,
        }
    }

    // coordinates of the center (left-aligned) of the bottom-most line of pieces spawned on this field
    fn piece_origin(&self) -> Coords {
        Coords(self.hidden as i32 - 2, self.width as i32 / 2 - 1)
    }

    fn set_at(&mut self, Coords(row, col): Coords, square: Square<P>) {
        *self.lines[row as usize].get_mut(col as usize) = square;
    }
}

impl<P: PieceKind> Field<P> for DefaultField<P> {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn hidden(&self) -> usize {
        self.hidden
    }

    fn lines(&self) -> &[Line<P>] {
        &self.lines
    }

    fn get_at(&self, Coords(row, col): Coords) -> Square<P> {
        self.lines[row as usize].get(col as usize)
    }

    fn cur_piece(&self) -> Option<&LivePiece<P>> {
        self.cur_piece.as_ref()
    }

    fn held_piece(&self) -> Option<&LivePiece<P>> {
        self.held_piece.as_ref()
    }

    fn spawn_piece(&mut self, bag: &mut impl Bag<P>) {
        let kind = bag.next(self);
        let piece = LivePiece::new(kind, self.piece_origin());

        for coords in piece.coords() {
            self.set_at(*coords, Square::Filled(piece.kind())); // TODO: make sure not blocked
        }
        self.cur_piece = Some(piece);
    }
}
