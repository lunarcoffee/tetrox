use crate::{
    canvas::{self, Field, HoldPiece, NextQueue},
    config::{Config, Input},
    util,
};

use std::{cell::RefCell, collections::HashMap};

use js_sys::Date;
use strum::IntoEnumIterator;
use sycamore::{
    component,
    generic_node::Html,
    motion::create_raf_loop,
    prelude::{
        create_effect, create_signal, provide_context, provide_context_ref, use_context, ReadSignal,
        Scope, Signal,
    },
    view,
    view::View,
};
use tetrox::{
    field::DefaultField,
    tetromino::{SrsKickTable, SrsTetromino, TetrIo180KickTable},
    PieceKind, SingleBag,
};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlImageElement, KeyboardEvent};

#[component]
pub fn Board<'a, P: PieceKind + 'static, G: Html>(cx: Scope<'a>) -> View<G> {
    let config = use_context::<Signal<RefCell<Config>>>(cx);
    let c = config.get();
    let c = c.borrow();

    let mut bag = SingleBag::<P>::new();
    let field = DefaultField::new(c.field_width, c.field_height, c.field_hidden, &mut bag);
    let field_signal = create_signal(cx, RefCell::new(field));
    provide_context_ref(cx, field_signal);

    // prop for next queue
    let bag = create_signal(cx, RefCell::new(bag));

    // update field on field dimension config option updates
    let field_dims = util::create_config_selector(cx, config, |c| (c.field_width, c.field_height, c.field_hidden));
    create_effect(cx, || {
        let (width, height, hidden) = *field_dims.get();
        let new_field = util::with_signal_mut_untracked(bag, |bag| DefaultField::new(width, height, hidden, bag));
        field_signal.set(RefCell::new(new_field));
    });

    // used in canvas drawing
    provide_context(cx, make_asset_cache());

    // board css style config options
    let style_values = util::create_config_selector(cx, config, |c| (c.field_zoom * 100.0, c.vertical_offset));
    let game_style = style_values.map(cx, |d| format!("transform: scale({}%); margin-top: {}px;", d.0, d.1));

    // loop timer durations
    let das_arr = util::create_config_selector(cx, config, |c| (c.delayed_auto_shift, c.auto_repeat_rate));
    let arr = das_arr.map(cx, |d| d.1);
    let sdr = util::create_config_selector(cx, config, |c| c.soft_drop_rate);

    let inputs = create_signal(cx, RefCell::new(InputStates::new()));

    // creates an action that moves the piece to be executed on every tick of a loop timer
    // special action is given for a delay of zero
    macro_rules! loop_timer_shift_action {
        ($rows:expr, $cols:expr, $delay:expr) => {
            $delay.map(cx, |delay| {
                RefCell::new(if *delay == 0 {
                    |field: &mut DefaultField<P>| while field.try_shift($rows, $cols) {}
                } else {
                    |field: &mut DefaultField<P>| drop(field.try_shift($rows, $cols))
                })
            })
        };
    }

    type TimerAction<P> = RefCell<impl FnMut(&mut DefaultField<P>) + Copy + Clone>;

    let left_action = loop_timer_shift_action!(0, -1, arr);
    let right_action = loop_timer_shift_action!(0, 1, arr);
    let soft_drop_action = loop_timer_shift_action!(1, 0, sdr);

    // timer loop executing an action on an interval
    let loop_timer = |duration: &'a ReadSignal<u32>, input, action: &'a ReadSignal<TimerAction<P>>| {
        // derive timer from looping interval
        let timer = duration.map(cx, move |d| Timer::new(cx, *d));

        create_effect(cx, move || {
            let timer = timer.get();
            if timer.is_finished() {
                timer.stop(); // reset for the next buffer timer activation

                let state = inputs.get_untracked().borrow().get_state(&input);
                if state.is_held() {
                    if state.is_pressed() {
                        util::with_signal_mut_untracked(field_signal, |field| action.get().borrow_mut()(field));
                    }
                    // continue the timer loop if the input is held (pressed or suppressed)
                    timer.start();
                }
            }
        });

        timer
    };

    // timer loop executing an action on an interval after an initial buffer timeout
    let buffered_loop_timer = |durations: &'a ReadSignal<(_, _)>, input, action: &'a ReadSignal<TimerAction<P>>| {
        // derive timers from buffer and loop durations
        let buffer_timer = durations.map(cx, move |d| Timer::new(cx, d.0));
        let loop_timer = loop_timer(durations.map(cx, |d| d.1), input, action);

        create_effect(cx, move || {
            let buffer_timer = buffer_timer.get();

            if buffer_timer.is_finished() {
                buffer_timer.stop(); // reset for the next buffer timer activation

                // apply the action if the input is still held down
                if inputs.get_untracked().borrow().get_state(&input).is_pressed() {
                    util::with_signal_mut_untracked(field_signal, |field| action.get().borrow_mut()(field));
                }
                loop_timer.get().start(); // activate the loop timer
            }
        });

        buffer_timer
    };

    // looping input timers
    let left_timer = buffered_loop_timer(das_arr, Input::Left, left_action);
    let right_timer = buffered_loop_timer(das_arr, Input::Right, right_action);
    let soft_drop_timer = loop_timer(sdr, Input::SoftDrop, soft_drop_action);

    let keydown_handler = |e: Event| {
        let e = e.dyn_into::<KeyboardEvent>().unwrap();
        let c = config.get();
        let c = c.borrow();

        c.inputs.get_by_right(&e.key()).map(|input| {
            // don't do anything if the input was already pressed
            // these presses come from the operating system repeating inputs automatically
            if util::with_signal_mut(inputs, |inputs| inputs.set_pressed(input)).is_pressed() {
                return;
            }

            util::with_signal_mut(field_signal, |field| {
                // shift the current piece and activate a loop timer to handle a held input
                let mut shift_and_start_timer = |rows, cols, timer: &ReadSignal<Timer>| {
                    field.try_shift(rows, cols);
                    timer.get().start();
                };

                match input {
                    Input::Left => shift_and_start_timer(0, -1, left_timer),
                    Input::Right => shift_and_start_timer(0, 1, right_timer),
                    Input::SoftDrop => shift_and_start_timer(1, 0, soft_drop_timer),
                    Input::HardDrop => drop(util::with_signal_mut_silent(bag, |bag| field.hard_drop(bag))),
                    Input::RotateCw => drop(field.try_rotate_cw(&SrsKickTable)),
                    Input::RotateCcw => drop(field.try_rotate_ccw(&SrsKickTable)),
                    Input::Rotate180 => drop(field.try_rotate_180(&TetrIo180KickTable)),
                    Input::SwapHoldPiece => util::with_signal_mut_silent(bag, |bag| field.swap_hold_piece(bag)),
                    _ => {}
                }
            });

            // handle game resetting separately as `with_signal_mut` will replace the new field with the old one
            // after the closure executes
            if input == &Input::Reset {
                let mut new_bag = SingleBag::new();
                let field = DefaultField::new(c.field_width, c.field_height, c.field_hidden, &mut new_bag);

                field_signal.set(RefCell::new(field));
                bag.set(RefCell::new(new_bag));
                inputs.set(RefCell::new(InputStates::new()));
            }

            // only notify bag subscribers after the field is updated
            // certain field updates (e.g. hard drop) also update the bag, which updates the next queue, which
            // requires a reference to the field (but `with_signal_mut` already has an exclusive reference)
            util::notify_subscribers(bag);
        });
    };

    let keyup_handler = |e: Event| {
        let e = e.dyn_into::<KeyboardEvent>().unwrap();
        let c = config.get();
        let c = c.borrow();

        c.inputs.get_by_right(&e.key()).map(|input| {
            util::with_signal_mut(inputs, |inputs| inputs.set_released(input));

            // cancel timers on release
            // this means pressing the input again before the buffer timer completes will not cause the action to run
            match input {
                Input::Left => left_timer.get().stop(),
                Input::Right => right_timer.get().stop(),
                Input::SoftDrop => soft_drop_timer.get().stop(),
                _ => {}
            }
        });
    };

    view! { cx,
        div(class="game", tabindex="0", style=game_style.get(), on:keydown=keydown_handler, on:keyup=keyup_handler) {
            div(class="field-panel") { div(class="hold-piece") { HoldPiece::<P, G> {} } }
            div(class="field") { Field::<P, G> {} }
            div(class="next-queue") { NextQueue { bag } }
        }
    }
}

pub type AssetCache = HashMap<String, HtmlImageElement>;

fn make_asset_cache() -> AssetCache {
    <SrsTetromino as PieceKind>::iter()
        .map(|k| k.asset_name().to_string())
        .chain(["grey".to_string()])
        .flat_map(|asset_name| {
            let field_square_mul = canvas::SQUARE_WIDTH as u32;
            crate::SKIN_NAMES.iter().map(move |skin| {
                let image = HtmlImageElement::new_with_width_and_height(field_square_mul, field_square_mul).unwrap();
                let asset_src = format!("assets/skins/{}/{}.png", skin, asset_name);
                image.set_src(&asset_src);
                (asset_src.to_string(), image)
            })
        })
        .collect()
}

// a resettable timer that waits for a timeout and sets a flag upon completion
struct Timer<'a>(RefCell<TimerInner<'a>>);

struct TimerInner<'a> {
    cx: Scope<'a>,

    duration: u32,
    is_finished: &'a Signal<bool>,
    stop_fn: Option<&'a dyn Fn()>,
}

impl<'a> Timer<'a> {
    fn new(cx: Scope<'a>, duration: u32) -> Self {
        Timer(RefCell::new(TimerInner {
            cx,

            duration,
            is_finished: create_signal(cx, false),
            stop_fn: None,
        }))
    }

    // this value is reactive and should be used to perform an action on completion of the timeout
    fn is_finished(&self) -> bool { *self.0.borrow().is_finished.get() }

    // run the timer, setting the `is_finished` signal to true when the `duration` has elapsed
    fn start(&self) {
        // stop the timer before starting it again
        self.stop();

        let end_time = Date::now() + self.0.borrow().duration as f64;
        let is_finished = self.0.borrow().is_finished.clone();

        // loop that keeps running until the specified duration has elapsed
        let (_, start, stop) = create_raf_loop(self.0.borrow().cx, move || {
            let keep_running = Date::now() < end_time;
            if !keep_running {
                is_finished.set(true);
            }
            keep_running
        });

        self.0.borrow_mut().stop_fn = Some(stop);
        start()
    }

    // stop any currently running timer and mark it as unfinished, effectively resetting it
    fn stop(&self) {
        self.0.borrow().stop_fn.map(|stop| stop());
        self.0.borrow().is_finished.set(false);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InputState {
    Released,
    Pressed,
    Suppressed,
}

impl InputState {
    pub fn is_pressed(&self) -> bool { self == &InputState::Pressed }

    pub fn is_held(&self) -> bool { self != &InputState::Released }
}

// states of all the `Input`s
pub struct InputStates {
    states: HashMap<Input, InputState>,
}

impl InputStates {
    fn new() -> Self {
        let states = Input::iter().map(|input| (input, InputState::Released)).collect();
        InputStates { states }
    }

    pub fn get_state(&self, input: &Input) -> InputState { *self.states.get(input).unwrap() }

    fn set_state(&mut self, input: &Input, state: InputState) -> InputState {
        self.states.insert(*input, state).unwrap()
    }

    pub fn set_pressed(&mut self, input: &Input) -> InputState {
        // if left or right, suppress the other if it is pressed
        if let Some(ref other) = Self::other_in_lr_pair(input) {
            if self.get_state(other) == InputState::Pressed {
                self.set_suppressed(other);
            }
        }
        self.set_state(input, InputState::Pressed)
    }

    pub fn set_released(&mut self, input: &Input) {
        // if left or right, unsuppress the other
        if let Some(ref other) = Self::other_in_lr_pair(input) {
            if self.get_state(other) == InputState::Suppressed {
                self.set_pressed(other);
            }
        }
        self.set_state(input, InputState::Released);
    }

    // suppressed inputs stop repeating until set to pressed or released
    fn set_suppressed(&mut self, input: &Input) { self.set_state(input, InputState::Suppressed); }

    // return the other input if the given input is left or right
    fn other_in_lr_pair(input: &Input) -> Option<Input> {
        match input {
            Input::Left => Some(Input::Right),
            Input::Right => Some(Input::Left),
            _ => None,
        }
    }
}
