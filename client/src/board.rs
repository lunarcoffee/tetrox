use crate::{
    canvas::{self, Field, HoldPiece, NextQueue},
    config::{Config, Input},
    util,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc, time::Duration};

use strum::IntoEnumIterator;
use sycamore::{
    component, easing,
    generic_node::Html,
    motion::{create_tweened_signal, Tweened},
    prelude::{
        create_effect, create_selector, create_signal, provide_context, provide_context_ref, use_context, ReadSignal,
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

pub type AssetCache = HashMap<String, HtmlImageElement>;

fn make_asset_cache() -> AssetCache {
    <SrsTetromino as PieceKind>::iter()
        .map(|k| k.asset_name().to_string())
        .chain(["grey".to_string()])
        .map(|asset_name| {
            let field_square_mul = canvas::SQUARE_WIDTH as u32;
            let image = HtmlImageElement::new_with_width_and_height(field_square_mul, field_square_mul).unwrap();

            let asset_src = format!("assets/skins/{}/{}.png", "tetrox", asset_name);
            image.set_src(&asset_src);

            (asset_src.to_string(), image)
        })
        .collect()
}

// used to select specific config options to update on as opposed to updating on every config value change, even if the
// updated value isn't used in a given computation
pub fn create_config_selector<'a, T, F>(cx: Scope<'a>, config: &'a Signal<Config>, mut op: F) -> &'a ReadSignal<T>
where
    T: PartialEq + 'a,
    F: FnMut(Rc<Config>) -> T + 'a,
{
    create_selector(cx, move || op(config.get()))
}

// for unbuffered inputs (e.g. soft drop; sdr loop)
pub fn create_loop_timer<'a, P: PieceKind + 'a>(
    cx: Scope<'a>,
    inputs: &'a Signal<RefCell<InputStates>>,
    duration: &'a ReadSignal<u32>,
    field: &'a Signal<RefCell<DefaultField<P>>>,
    input: Input,
    mut action: impl FnMut(&mut DefaultField<P>) + Copy + Clone + 'a,
) -> &'a Tweened<'a, f64> {
    // TODO: make these update for updated das/arr with an effect?
    let duration = Duration::from_millis((*duration.get()).into());
    let timer = create_tweened_signal(cx, 1.0f64, duration, easing::linear);

    create_effect(cx, move || {
        // if the value has finished tweening from 1.0 to 0.0 (i.e. the timeout has elapsed)
        if !timer.is_tweening() && *timer.get() == 0.0 {
            timer.signal().set(1.0); // reset for the next timer activation

            let state = inputs.get_untracked().borrow().get_state(&input);
            if state.is_held() {
                if state.is_pressed() {
                    util::with_signal_mut_untracked(field, |field| action(field));
                }
                // continue the timer loop if the input is held (pressed or suppressed)
                timer.set(0.0);
            }
        }
    });

    timer
}

// for buffered inputs (e.g. left/right movement; das buffer + arr loop)
pub fn create_buffered_loop_timer<'a, P: PieceKind + 'a>(
    cx: Scope<'a>,
    inputs: &'a Signal<RefCell<InputStates>>,
    durations: &'a ReadSignal<(u32, u32)>,
    field: &'a Signal<RefCell<DefaultField<P>>>,
    input: Input,
    mut action: impl FnMut(&mut DefaultField<P>) + Copy + Clone + 'a,
) -> &'a Tweened<'a, f64> {
    // TODO: make these update for updated das/arr with an effect?
    let buffer_duration = Duration::from_millis(durations.get().0.into());
    let buffer_timer = create_tweened_signal(cx, 1.0f64, buffer_duration, easing::linear);

    let loop_duration = durations.map(cx, |(_, b)| *b);
    let loop_timer = create_loop_timer(cx, inputs, loop_duration, field, input, action);

    // buffered timer which activates loop timer after an initial buffer time
    create_effect(cx, move || {
        if !buffer_timer.is_tweening() && *buffer_timer.get() == 0.0 {
            // apply the action if the input is still held down
            if inputs.get_untracked().borrow().get_state(&input).is_pressed() {
                util::with_signal_mut_untracked(field, |field| action(field));
            }
            loop_timer.set(0.0); // activate the loop timer
            buffer_timer.signal().set(1.0); // reset for the next buffer timer activation
        }
    });

    buffer_timer
}

#[component]
pub fn Board<'a, P: PieceKind + 'static, G: Html>(cx: Scope<'a>) -> View<G> {
    let config = use_context::<Signal<Config>>(cx);
    let c = config.get();

    let mut bag = SingleBag::<P>::new();
    let field = DefaultField::new(c.field_width, c.field_height, c.field_hidden, &mut bag);
    let field_signal = create_signal(cx, RefCell::new(field));
    provide_context_ref(cx, field_signal);

    // prop for next queue
    let bag = create_signal(cx, RefCell::new(bag));

    // update field on field dimension config option updates
    let field_dims = create_config_selector(cx, config, |c| (c.field_width, c.field_height, c.field_hidden));
    create_effect(cx, || {
        let (width, height, hidden) = *field_dims.get();
        let new_field = util::with_signal_mut_untracked(bag, |bag| DefaultField::new(width, height, hidden, bag));
        field_signal.set(RefCell::new(new_field));
    });

    // used in canvas drawing
    provide_context(cx, make_asset_cache());

    // board css style config options
    let style_values = create_config_selector(cx, config, |c| (c.field_zoom * 100.0, c.vertical_offset));
    let game_style = style_values.map(cx, |d| format!("transform: scale({}%); margin-top: {}px;", d.0, d.1));

    // loop timer durations
    let das_arr = create_config_selector(cx, config, |c| (c.delayed_auto_shift, c.auto_repeat_rate));
    let sdr = create_config_selector(cx, config, |c| c.soft_drop_rate);

    let inputs = create_signal(cx, RefCell::new(InputStates::new()));
    let shift = |rows, cols| move |field: &mut DefaultField<P>| drop(field.try_shift(rows, cols));

    // looping input timers
    let left_timer = create_buffered_loop_timer(cx, inputs, das_arr, field_signal, Input::Left, shift(0, -1));
    let right_timer = create_buffered_loop_timer(cx, inputs, das_arr, field_signal, Input::Right, shift(0, 1));
    let soft_drop_timer = create_loop_timer(cx, inputs, sdr, field_signal, Input::SoftDrop, shift(1, 0));

    let keydown_handler = |e: Event| {
        let e = e.dyn_into::<KeyboardEvent>().unwrap();
        let config = config.get();

        config.inputs.get_by_right(&e.key()).map(|input| {
            // do action if the input wasn't already pressed
            let prev_state = util::with_signal_mut(inputs, |inputs| inputs.set_pressed(input));
            if !prev_state.is_pressed() {
                util::with_signal_mut(field_signal, |field| match input {
                    Input::Left => {
                        field.try_shift(0, -1);
                        left_timer.set(0.0);
                    }
                    Input::Right => {
                        field.try_shift(0, 1);
                        right_timer.set(0.0);
                    }
                    Input::SoftDrop => {
                        // field.project_down();
                        field.try_shift(1, 0);
                        soft_drop_timer.set(0.0);
                    } // TODO:
                    Input::HardDrop => drop(util::with_signal_mut_silent(bag, |bag| field.hard_drop(bag))),
                    Input::RotateCw => drop(field.try_rotate_cw(&SrsKickTable)),
                    Input::RotateCcw => drop(field.try_rotate_ccw(&SrsKickTable)),
                    Input::Rotate180 => drop(field.try_rotate_180(&TetrIo180KickTable)),
                    // Input::SwapHoldPiece => todo!(),
                    // Input::Reset => todo!(),
                    // Input::ShowHideUi => todo!(),
                    _ => {}
                });

                // only notify bag subscribers after the field is updated
                // certain field updates (e.g. hard drop) also update the bag, which updates the next queue, which requires
                // a reference to the field (but `with_signal_mut` already has an exclusive reference)
                util::notify_subscribers(bag);
            }
        });
    };

    let keyup_handler = |e: Event| {
        let e = e.dyn_into::<KeyboardEvent>().unwrap();
        let config = config.get();
        let keybind = config.inputs.get_by_right(&e.key());
        keybind.map(|input| util::with_signal_mut(inputs, |inputs| inputs.set_released(input)));
    };

    view! { cx,
        div(class="game", tabindex="0", style=(game_style.get()), on:keydown=keydown_handler, on:keyup=keyup_handler) {
            div(class="field-panel") { div(class="hold-piece") { HoldPiece::<P, G> {} } }
            div(class="field") { Field::<P, G> {} }
            div(class="next-queue") { NextQueue { bag } }
        }
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
