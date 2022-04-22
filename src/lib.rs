#![feature(array_chunks)]
#![feature(min_specialization)]
#![feature(type_alias_impl_trait)]

pub mod field;
pub mod pieces;

use std::{mem, ops};

use field::DefaultField;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use pieces::{mino123::Mino123, tetromino::TetrominoSrs};
use rand::prelude::SliceRandom;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Coords(pub i32, pub i32);

impl Coords {
    pub fn to_coords_float(self) -> CoordsFloat { CoordsFloat(self.0 as f64, self.1 as f64) }
}

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

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CoordsFloat(pub f64, pub f64);

impl CoordsFloat {
    pub fn zero() -> CoordsFloat { CoordsFloat(0., 0.) }

    pub fn to_coords(self) -> Coords { Coords(self.0 as i32, self.1 as i32) }
}

impl ops::Add for CoordsFloat {
    type Output = CoordsFloat;

    fn add(self, CoordsFloat(row2, col2): Self) -> Self::Output { CoordsFloat(self.0 + row2, self.1 + col2) }
}

impl ops::Sub for CoordsFloat {
    type Output = CoordsFloat;

    fn sub(self, CoordsFloat(row2, col2): Self) -> Self::Output { CoordsFloat(self.0 - row2, self.1 - col2) }
}

pub trait PieceKindTrait {
    // coords of the squares composing the piece relative to the spawn coords
    fn spawn_offsets(&self) -> Vec<Coords>;

    // index of the rotation pivot of the piece with a possibly zero offset
    // pieces like the i tetromino have apparent pivots that intersect
    fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat);

    // returns the type of spin after a hard drop (if any) and whether it is mini
    fn detect_spin(&self, field: &DefaultField) -> (Option<PieceKind>, bool);

    fn asset_name(&self) -> &str;

    // iterator through all piece kinds
    fn iter() -> Box<dyn Iterator<Item = PieceKind>>;

    fn n_kinds() -> usize;
}

#[derive(Copy, Clone, Debug)]
pub enum PieceKind {
    TetrominoSrs(TetrominoSrs),
    Mino123(Mino123),
}

// generate match statement over all `PieceKind`s that calls a method, optionally with arguments
macro_rules! gen_piece_kind_match {
    ($self:ident, $method:ident $(,)? $($arg:expr),*) => { match $self {
        PieceKind::TetrominoSrs(p) => p.$method($($arg,)*),
        PieceKind::Mino123(p) => p.$method($($arg,)*),
    } }
}

// same as above but for associated functions
macro_rules! gen_piece_kind_match_associated {
    ($self:ident, $method:ident $(,)? $($arg:expr),*) => { match $self {
        PieceKind::TetrominoSrs(_) => TetrominoSrs::$method($($arg,)*),
        PieceKind::Mino123(_) => Mino123::$method($($arg,)*),
    } }
}

impl PieceKind {
    pub fn spawn_offsets(&self) -> Vec<Coords> { gen_piece_kind_match!(self, spawn_offsets) }

    pub fn pivot_offset(&self, rotation_state: RotationState) -> (usize, CoordsFloat) {
        gen_piece_kind_match!(self, pivot_offset, rotation_state)
    }

    pub fn detect_spin(&self, field: &DefaultField) -> (Option<Self>, bool) {
        gen_piece_kind_match!(self, detect_spin, field)
    }

    pub fn asset_name(&self) -> &str { gen_piece_kind_match!(self, asset_name) }

    pub fn iter(&self) -> Box<dyn Iterator<Item = PieceKind>> { gen_piece_kind_match_associated!(self, iter) }

    pub fn n_kinds(&self) -> usize { gen_piece_kind_match_associated!(self, n_kinds) }
}

pub trait Randomizer {
    fn next(&mut self) -> PieceKind;
    fn peek(&mut self) -> Box<dyn Iterator<Item = PieceKind> + '_>;

    fn lookahead(&self) -> usize;
}

pub struct SingleBag {
    kinds: Vec<PieceKind>,
    bag: Vec<PieceKind>,
}

impl SingleBag {
    pub fn new(kinds: Vec<PieceKind>) -> Self {
        let mut bag = SingleBag { kinds, bag: vec![] };
        bag.update_bag();
        bag.update_bag();
        bag
    }

    fn update_bag(&mut self) {
        if self.bag.len() <= self.kinds.len() {
            let mut next_bag = self.kinds.clone();
            next_bag.shuffle(&mut rand::thread_rng());

            // prepend to preserve peek order
            mem::swap(&mut self.bag, &mut next_bag);
            self.bag.extend(next_bag);
        }
    }
}

impl Randomizer for SingleBag {
    fn next(&mut self) -> PieceKind {
        self.update_bag();
        self.bag.pop().unwrap()
    }

    fn peek(&mut self) -> Box<dyn Iterator<Item = PieceKind> + '_> {
        self.update_bag();
        Box::new(self.bag.iter().rev().cloned())
    }

    fn lookahead(&self) -> usize { self.kinds.len() }
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

pub trait KickTable {
    fn rotate_cw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords>;
    fn rotate_ccw(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords>;
}

pub trait KickTable180 {
    fn rotate_180(&self, piece: PieceKind, rotation_state: RotationState) -> Vec<Coords>;
}
