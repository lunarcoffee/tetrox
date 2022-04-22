#![feature(array_chunks)]
#![feature(min_specialization)]
#![feature(type_alias_impl_trait)]

pub mod field;
pub mod pieces;
pub mod kicks;

use std::{mem, ops};

use pieces::PieceKind;
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
