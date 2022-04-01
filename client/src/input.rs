use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gloo_timers::callback::{Interval, Timeout};
use strum::IntoEnumIterator;
use yew::{html::Scope, Context};

use crate::{
    board::{Board, BoardMessage},
    config::{ReadOnlyConfig, Input},
};

// timeout for das, intervals for arr and soft dropping
enum MoveTimer {
    Timeout(Timeout),
    Interval(Interval),
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
    states: Rc<RefCell<HashMap<Input, InputState>>>,
    timers: Vec<(Input, MoveTimer)>,

    config: ReadOnlyConfig,
}

impl InputStates {
    pub fn new(config: ReadOnlyConfig) -> Self {
        let map = Input::iter().map(|input| (input, InputState::Released)).collect();
        let states = Rc::new(RefCell::new(map));

        InputStates {
            states,
            timers: vec![],
            config,
        }
    }

    fn get_state(&mut self, input: Input) -> InputState { *self.states.clone().borrow().get(&input).unwrap() }

    fn set_state(&mut self, input: Input, state: InputState) { self.states.clone().borrow_mut().insert(input, state); }

    pub fn is_pressed(&self, input: Input) -> bool {
        self.states.clone().borrow().get(&input).unwrap() == &InputState::Pressed
    }

    fn set_pressed(&mut self, input: Input) { self.set_state(input, InputState::Pressed); }

    // this acts as a guard to make sure repeated inputs don't trigger additional actions once an input is pressed
    pub fn set_pressed_msg(&mut self, input: Input, ctx: &Context<Board>, message: BoardMessage) {
        let input_down = self.is_pressed(input);
        if !input_down {
            self.set_pressed(input);
            ctx.link().send_message(message);
        }
    }

    // this unsets the guard, cancelling any active timers and re-enabling the action
    pub fn set_released(&mut self, input: Input) {
        // if left or right, unsuppress the other
        if let Some(other) = Self::other_in_lr_pair(input) {
            if self.get_state(other) == InputState::Suppressed {
                self.set_pressed(other);
            }
        }

        self.set_state(input, InputState::Released);
        self.timers.retain(|t| t.0 != input);
    }

    // this will cause the suppressed held input to stop repeating until set to pressed or released
    pub fn set_suppressed(&mut self, input: Input) { self.set_state(input, InputState::Suppressed); }

    // wait for das if the left input isn't already pressed
    pub fn left_pressed(&mut self, ctx: &Context<Board>) {
        let left_down = self.is_pressed(Input::Left);
        if !left_down {
            self.set_pressed(Input::Left);

            let link = ctx.link().clone();
            link.send_message(BoardMessage::MoveLeft);

            let timeout = Timeout::new(self.config.delayed_auto_shift, move || {
                link.send_message(BoardMessage::MoveLeftAutoRepeat);
            });
            self.timers.push((Input::Left, MoveTimer::Timeout(timeout)));
        }
    }

    pub fn right_pressed(&mut self, ctx: &Context<Board>) {
        let right_down = self.is_pressed(Input::Right);
        if !right_down {
            self.set_pressed(Input::Right);

            let link = ctx.link().clone();
            link.send_message(BoardMessage::MoveRight);

            let timeout = Timeout::new(self.config.delayed_auto_shift, move || {
                link.send_message(BoardMessage::MoveRightAutoRepeat);
            });
            self.timers.push((Input::Right, MoveTimer::Timeout(timeout)));
        }
    }

    // keep shifting down while the soft drop input is pressed
    pub fn soft_drop_pressed(&mut self, ctx: &Context<Board>) {
        let soft_drop_rate = self.config.soft_drop_rate;

        let soft_drop_down = self.is_pressed(Input::SoftDrop);
        if !soft_drop_down {
            self.set_pressed(Input::SoftDrop);

            let link = ctx.link().clone();
            let action = move || {
                Self::send_message_or_if_zero(&link, soft_drop_rate, BoardMessage::ProjectDown, BoardMessage::MoveDown)
            };
            action();

            let interval = Interval::new(soft_drop_rate, action);
            self.timers.push((Input::SoftDrop, MoveTimer::Interval(interval)));
        }
    }

    // keep shifting left while the left input is pressed
    pub fn left_held(&mut self, ctx: &Context<Board>) {
        let auto_repeat_rate = self.config.auto_repeat_rate;

        let states = self.states.clone();
        let link = ctx.link().clone();
        let action = move || {
            if states.borrow().get(&Input::Left).unwrap() == &InputState::Pressed {
                Self::send_message_or_if_zero(&link, auto_repeat_rate, BoardMessage::DasLeft, BoardMessage::MoveLeft)
            }
        };
        action();

        let interval = Interval::new(auto_repeat_rate, action);
        self.timers.push((Input::Left, MoveTimer::Interval(interval)));
    }

    pub fn right_held(&mut self, ctx: &Context<Board>) {
        let auto_repeat_rate = self.config.auto_repeat_rate;

        let states = self.states.clone();
        let link = ctx.link().clone();
        let action = move || {
            if states.borrow().get(&Input::Right).unwrap() == &InputState::Pressed {
                Self::send_message_or_if_zero(&link, auto_repeat_rate, BoardMessage::DasRight, BoardMessage::MoveRight)
            }
        };
        action();

        let interval = Interval::new(auto_repeat_rate, action);
        self.timers.push((Input::Right, MoveTimer::Interval(interval)));
    }

    // send a message (probably movement) to the board with special behavior for zero (e.g. das, arr, etc.)
    fn send_message_or_if_zero(
        link: &Scope<Board>,
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

    pub fn update_config(&mut self, config: ReadOnlyConfig) { self.config = config; }
}
