use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use gloo_timers::callback::{Interval, Timeout};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew::{html::Scope, Context};

use crate::{BoardMessage, BoardModel};

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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputState {
    Released,
    Pressed,
    // other input of pair (e.g. left and right) pressed while the other is already pressed
    // allows the latest input to take precedence
    Suppressed,
}

// states (pressed/released) of all inputs
pub struct InputStates {
    states: Arc<Mutex<HashMap<Input, InputState>>>,
    timers: Vec<(Input, MoveTimer)>,
}

// das, arr, sdr in milliseconds
pub const DELAYED_AUTO_SHIFT: u32 = 120;
pub const AUTO_REPEAT_RATE: u32 = 0;
pub const SOFT_DROP_RATE: u32 = 0;

impl InputStates {
    pub fn new() -> Self {
        let map = Input::iter().map(|input| (input, InputState::Released)).collect();
        let states = Arc::new(Mutex::new(map));
        InputStates { states, timers: vec![] }
    }

    fn get_state(&mut self, input: Input) -> InputState { *self.states.clone().lock().unwrap().get(&input).unwrap() }

    fn set_state(&mut self, input: Input, state: InputState) {
        self.states.clone().lock().unwrap().insert(input, state);
    }
    
    pub fn is_pressed(&self, input: Input) -> bool {
        self.states.clone().lock().unwrap().get(&input).unwrap() == &InputState::Pressed
    }

    fn set_pressed(&mut self, input: Input) { self.set_state(input, InputState::Pressed); }

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

    // this unsets the guard, cancelling any active timers and re-enabling the action
    pub fn set_released(&mut self, input: Input) -> bool {
        // if left or right, unsuppress the other
        if let Some(other) = Self::other_in_lr_pair(input) {
            if self.get_state(other) == InputState::Suppressed {
                self.set_pressed(other);
            }
        }

        self.set_state(input, InputState::Released);
        self.timers.retain(|t| t.0 != input);
        false
    }

    // this will cause the suppressed held input to stop repeating until set to pressed or released
    pub fn set_suppressed(&mut self, input: Input) -> bool {
        self.set_state(input, InputState::Suppressed);
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
            self.timers.push((Input::Left, MoveTimer::Timeout(timeout)));
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
            self.timers.push((Input::Right, MoveTimer::Timeout(timeout)));
        }
        !right_down
    }

    // keep shifting down while the soft drop input is pressed
    pub fn soft_drop_pressed(&mut self, ctx: &Context<BoardModel>) -> bool {
        let soft_drop_down = self.is_pressed(Input::SoftDrop);
        if !soft_drop_down {
            self.set_pressed(Input::SoftDrop);

            let link = ctx.link().clone();
            let action = move || {
                Self::send_message_or_if_zero(&link, SOFT_DROP_RATE, BoardMessage::ProjectDown, BoardMessage::MoveDown)
            };
            action();

            let interval = Interval::new(SOFT_DROP_RATE, action);
            self.timers.push((Input::SoftDrop, MoveTimer::Interval(interval)));
        }
        !soft_drop_down
    }

    // keep shifting left while the left input is pressed
    pub fn left_held(&mut self, ctx: &Context<BoardModel>) -> bool {
        let states = self.states.clone();
        let link = ctx.link().clone();
        let action = move || {
            if states.lock().unwrap().get(&Input::Left).unwrap() == &InputState::Pressed {
                Self::send_message_or_if_zero(&link, AUTO_REPEAT_RATE, BoardMessage::DasLeft, BoardMessage::MoveLeft)
            }
        };
        action();

        let interval = Interval::new(AUTO_REPEAT_RATE, action);
        self.timers.push((Input::Left, MoveTimer::Interval(interval)));

        false
    }

    pub fn right_held(&mut self, ctx: &Context<BoardModel>) -> bool {
        let states = self.states.clone();
        let link = ctx.link().clone();
        let action = move || {
            if states.lock().unwrap().get(&Input::Right).unwrap() == &InputState::Pressed {
                Self::send_message_or_if_zero(&link, AUTO_REPEAT_RATE, BoardMessage::DasRight, BoardMessage::MoveRight)
            }
        };
        action();

        let interval = Interval::new(AUTO_REPEAT_RATE, action);
        self.timers.push((Input::Right, MoveTimer::Interval(interval)));

        false
    }

    // send a message (probably movement) to the board with special behavior for zero (e.g. das, arr, etc.)
    fn send_message_or_if_zero(
        link: &Scope<BoardModel>,
        value: u32,
        message_nonzero: BoardMessage,
        message_zero: BoardMessage,
    ) {
        if value == 0 {
            link.send_message(message_nonzero);
        } else {
            link.send_message(message_zero);
        }
    }

    // return the other input if the given input is left or right
    fn other_in_lr_pair(input: Input) -> Option<Input> {
        match input {
            Input::Left => Some(Input::Right),
            Input::Right => Some(Input::Left),
            _ => None,
        }
    }
}
