use crate::{
    canvas::{self, Field, HoldPiece, NextQueue},
    config::{Config, Input},
    util,
};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use strum::IntoEnumIterator;
use sycamore::{
    component,
    generic_node::Html,
    prelude::{create_signal, provide_context, provide_context_ref, use_context, Scope, Signal},
    view,
    view::View,
};
use tetrox::{
    field::DefaultField,
    tetromino::{SrsKickTable, SrsTetromino, Tetrio180KickTable},
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

#[component]
pub fn Board<'a, P: PieceKind + 'static, G: Html>(cx: Scope<'a>) -> View<G> {
    let config_signal = use_context::<Signal<Config>>(cx); // TODO: does calling get on this not react
    let config = config_signal.get();

    let mut bag = SingleBag::<P>::new();
    let field = DefaultField::new(config.field_width, config.field_height, config.field_hidden, &mut bag);
    let field = create_signal(cx, RefCell::new(field));
    provide_context_ref(cx, field);

    let bag = create_signal(cx, RefCell::new(bag));

    // cached image elements used in canvas drawing
    let asset_cache = make_asset_cache();
    provide_context(cx, asset_cache);

    // let input_timers = cx.create_signal(vec![]);
    let inputs_signal = create_signal(cx, RefCell::new(InputHandler::new()));
    // cx.create_effect(|| {
    //     let inputs = inputs_signal.get();
    //     let inputs = inputs.borrow();

    //     // TODO: if button is now held, launch timer?
    // });

    // board style config options
    let scale = config.field_zoom * 100.0;
    let margin = config.vertical_offset;
    let game_style = format!("transform: scale({}%); margin-top: {}px;", scale, margin);

    view! { cx,
        div(
            class="game",
            tabindex="0",
            style=(game_style),
            on:keydown=|e: Event| {
                let e = e.dyn_into::<KeyboardEvent>().unwrap();
                let config = config_signal.get();

                web_sys::console::log_1(&format!("key: {:?}", e.key()).into());
                
                config.inputs.get_by_right(&e.key()).map(|input| {
                    util::with_signal_mut(inputs_signal, |inputs| {
                        // do action if the input wasn't already pressed
                        if !inputs.set_pressed(input).is_pressed() {
                            util::with_signal_mut(field, |field| match *input {
                                Input::Left => field.try_shift(0, -1),
                                Input::Right => field.try_shift(0, 1),
                                // Input::SoftDrop => todo!(),
                                // Input::HardDrop => todo!(),
                                // Input::RotateCw => field.try_rotate_cw(&SrsKickTable),
                                // Input::RotateCcw => field.try_rotate_ccw(&SrsKickTable),
                                // Input::Rotate180 => field.try_rotate_180(&Tetrio180KickTable),
                                // Input::SwapHoldPiece => todo!(),
                                // Input::Reset => todo!(),
                                // Input::ShowHideUi => todo!(),
                                _ => false,
                            });
                        }
                    });
                });
            },
            on:keyup=|e: Event| {
                let e = e.dyn_into::<KeyboardEvent>().unwrap();
                let config = config_signal.get();
                config
                    .inputs
                    .get_by_right(&e.key())
                    .map(|input| util::with_signal_mut(inputs_signal, |inputs| inputs.set_released(input)));
            },
        ) {
            div(class="field-panel") { div(class="hold-piece") { HoldPiece::<P, G> {} } }
            div(class="field") { Field::<P, G> {} }
            div(class="next-queue") { NextQueue { bag } }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum InputState {
    Released,
    Pressed,
    Suppressed,
}

impl InputState {
    pub fn is_pressed(&self) -> bool { self == &InputState::Pressed }
}

pub struct InputHandler {
    states: Rc<RefCell<HashMap<Input, InputState>>>,
}

impl InputHandler {
    fn new() -> Self {
        let initial = Input::iter().map(|input| (input, InputState::Released)).collect();
        let states = Rc::new(RefCell::new(initial));

        InputHandler { states }
    }

    fn get_state(&self, input: &Input) -> InputState { *self.states.clone().borrow().get(input).unwrap() }

    fn set_state(&mut self, input: &Input, state: InputState) -> InputState {
        self.states.clone().borrow_mut().insert(*input, state).unwrap()
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

    // this unsets the guard, cancelling any active timers and re-enabling the action
    pub fn set_released(&mut self, input: &Input) {
        // if left or right, unsuppress the other
        if let Some(ref other) = Self::other_in_lr_pair(input) {
            if self.get_state(other) == InputState::Suppressed {
                self.set_pressed(other);
            }
        }
        self.set_state(input, InputState::Released);
    }

    // this will cause the suppressed held input to stop repeating until set to pressed or released
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
