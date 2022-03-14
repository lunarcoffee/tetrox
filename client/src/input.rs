use std::collections::HashMap;

use gloo_timers::callback::{Interval, Timeout};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew::{html::Scope, Context};

use crate::{BoardMessage, BoardModel};

// used to determine what timers to cancel when movement keys are released
#[derive(PartialEq, Eq)]
enum MoveDirection {
    Left,
    Right,
    Down,
}

// timeout for das, intervals for arr and soft dropping
enum MoveTimer {
    Timeout(Timeout),
    Interval(Interval),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Input {
    Left,
    Right,
    SoftDrop,
    HardDrop,
    RotateCw,
    RotateCcw,
    Rotate180,
}

// states (pressed/released) of all inputs
pub struct InputStates {
    pressed: HashMap<Input, bool>,
    timers: Vec<(MoveDirection, MoveTimer)>,
}

// das, arr, sdr in milliseconds
pub const DELAYED_AUTO_SHIFT: u32 = 120;
pub const AUTO_REPEAT_RATE: u32 = 0;
pub const SOFT_DROP_RATE: u32 = 0;

impl InputStates {
    pub fn new() -> Self {
        InputStates {
            pressed: Input::iter().map(|input| (input, false)).collect(),
            timers: vec![],
        }
    }

    pub fn is_pressed(&self, input: Input) -> bool { *self.pressed.get(&input).unwrap() }

    fn set_pressed(&mut self, input: Input) { self.pressed.insert(input, true); }

    // this acts as a guard to make sure repeated inputs don't trigger additional actions once an input is pressed
    pub fn set_pressed_with_action(&mut self, input: Input, action: impl FnOnce() -> bool) -> bool {
        let input_down = self.is_pressed(input);
        if !input_down {
            self.set_pressed(input);
            action()
        } else {
            false
        }
    }

    // this unsets the guard, re-enabling the action
    pub fn set_released(&mut self, input: Input) -> bool {
        self.pressed.insert(input, false);
        false
    }

    // wait for das if the left input isn't already pressed
    pub fn left_pressed(&mut self, ctx: &Context<BoardModel>) -> bool {
        let left_down = self.is_pressed(Input::Left);
        if !left_down {
            self.set_pressed(Input::Left);

            let link = ctx.link().clone();
            link.send_message(BoardMessage::MoveLeft);

            let timeout = Timeout::new(DELAYED_AUTO_SHIFT, move || {
                link.send_message(BoardMessage::MoveLeftAutoRepeat);
            });
            self.timers.push((MoveDirection::Left, MoveTimer::Timeout(timeout)));
        }
        !left_down
    }

    pub fn right_pressed(&mut self, ctx: &Context<BoardModel>) -> bool {
        let right_down = self.is_pressed(Input::Right);
        if !right_down {
            self.set_pressed(Input::Right);

            let link = ctx.link().clone();
            link.send_message(BoardMessage::MoveRight);

            let timeout = Timeout::new(DELAYED_AUTO_SHIFT, move || {
                link.send_message(BoardMessage::MoveRightAutoRepeat);
            });
            self.timers.push((MoveDirection::Right, MoveTimer::Timeout(timeout)));
        }
        !right_down
    }

    // keep shifting down while the soft drop input is pressed
    pub fn soft_drop_pressed(&mut self, ctx: &Context<BoardModel>) -> bool {
        let soft_drop_down = self.is_pressed(Input::SoftDrop);
        if !soft_drop_down {
            self.set_pressed(Input::SoftDrop);

            let link = ctx.link().clone();
            Self::send_soft_drop_move_message(&link);

            let interval = Interval::new(SOFT_DROP_RATE, move || Self::send_soft_drop_move_message(&link));
            self.timers.push((MoveDirection::Down, MoveTimer::Interval(interval)));
        }
        !soft_drop_down
    }

    // keep shifting left while the left input is pressed
    pub fn left_held(&mut self, ctx: &Context<BoardModel>) -> bool {
        let link = ctx.link().clone();
        Self::send_left_hold_move_message(&link);

        let interval = Interval::new(AUTO_REPEAT_RATE, move || Self::send_left_hold_move_message(&link));
        self.timers.push((MoveDirection::Left, MoveTimer::Interval(interval)));

        false
    }

    pub fn right_held(&mut self, ctx: &Context<BoardModel>) -> bool {
        let link = ctx.link().clone();
        Self::send_right_hold_move_message(&link);

        let interval = Interval::new(AUTO_REPEAT_RATE, move || Self::send_right_hold_move_message(&link));
        self.timers.push((MoveDirection::Right, MoveTimer::Interval(interval)));

        false
    }

    // cancel all timers contingent on left being pressed
    pub fn left_released(&mut self) -> bool {
        self.set_released(Input::Left);
        self.timers.retain(|t| !matches!(t, (MoveDirection::Left, ..)));

        false
    }

    pub fn right_released(&mut self) -> bool {
        self.set_released(Input::Right);
        self.timers.retain(|t| !matches!(t, (MoveDirection::Right, ..)));

        false
    }

    pub fn soft_drop_released(&mut self) -> bool {
        self.set_released(Input::SoftDrop);
        self.timers.retain(|t| !matches!(t, (MoveDirection::Down, ..)));
        false
    }

    // move or project the piece down based on the sdr
    pub fn send_soft_drop_move_message(link: &Scope<BoardModel>) {
        if SOFT_DROP_RATE == 0 {
            link.send_message(BoardMessage::ProjectDown);
        } else {
            link.send_message(BoardMessage::MoveDown);
        }
    }

    // move or das the piece left based on the arr
    pub fn send_left_hold_move_message(link: &Scope<BoardModel>) {
        if AUTO_REPEAT_RATE == 0 {
            link.send_message(BoardMessage::DasLeft)
        } else {
            link.send_message(BoardMessage::MoveLeft);
        }
    }

    pub fn send_right_hold_move_message(link: &Scope<BoardModel>) {
        if AUTO_REPEAT_RATE == 0 {
            link.send_message(BoardMessage::DasRight)
        } else {
            link.send_message(BoardMessage::MoveRight);
        }
    }
}
