use crate::{
    canvas::{self, Field, HoldPiece, NextQueue},
    config::{Config, GoalTypes, Input, SpinTypes, UiEnabled},
    goal,
    stats::Stats,
    timer::{self, Timer},
    util,
};

use std::{cell::RefCell, collections::HashMap};

use js_sys::Date;
use strum::IntoEnumIterator;
use sycamore::{
    component,
    generic_node::Html,
    prelude::{
        create_effect, create_selector, create_signal, provide_context, provide_context_ref, use_context, ReadSignal,
        Scope, Signal,
    },
    view,
    view::View,
};
use tetrox::{
    field::{DefaultField, LineClear},
    pieces::{tetromino::TetrominoSrs, PieceKindTrait},
    Randomizer, SingleBag,
};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlImageElement, KeyboardEvent};

#[component]
pub fn Board<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let config = use_context::<Signal<RefCell<Config>>>(cx);
    let c = config.get();
    let c = (*c.borrow()).clone();

    let piece_type = util::create_config_selector(cx, config, |c| c.piece_type);
    let piece_kinds = piece_type.get().kinds();
    let spin_types = util::create_config_selector(cx, config, |c| c.spin_types);

    let mut bag = SingleBag::new(piece_kinds.clone());
    let field = DefaultField::new(c.field_width, c.field_height, c.field_hidden, &piece_kinds, &mut bag);
    let field_signal = create_signal(cx, RefCell::new(field));
    provide_context_ref(cx, field_signal);

    let piece_kinds = piece_type.map(cx, |t| t.kinds());
    let bag = create_signal(cx, RefCell::new(bag));

    create_effect(cx, || {
        bag.set(RefCell::new(SingleBag::new((*piece_kinds.get()).clone())))
    });

    // update field on field dimension config option updates
    let field_dims = util::create_config_selector(cx, config, |c| (c.field_width, c.field_height, c.field_hidden));
    create_effect(cx, || {
        let (width, height, hidden) = *field_dims.get();
        let new_field = util::with_signal_mut_untracked(bag, |bag| {
            DefaultField::new(width, height, hidden, &*piece_kinds.get_untracked(), bag)
        });
        field_signal.set(RefCell::new(new_field));
    });

    // used in canvas drawing
    provide_context(cx, make_asset_cache());

    let time_elapsed = create_signal(cx, 0.0);
    provide_context_ref(cx, time_elapsed);

    // measuring time elapsed since last board reset
    let start_time = create_signal(cx, Date::now());
    let elapsed_timer = create_signal(cx, Timer::new(cx, 33));
    timer::create_timer_finish_effect(cx, elapsed_timer, move || {
        time_elapsed.set(Date::now() - *start_time.get());
        true
    });

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
                if *delay == 0 {
                    |field: &mut DefaultField| while field.try_shift($rows, $cols) {}
                } else {
                    |field: &mut DefaultField| drop(field.try_shift($rows, $cols))
                }
            })
        };
    }
    let left_action = loop_timer_shift_action!(0, -1, arr);
    let right_action = loop_timer_shift_action!(0, 1, arr);
    let soft_drop_action = loop_timer_shift_action!(1, 0, sdr);

    // timer loop executing an action on an interval
    let loop_timer = |delay: &'a ReadSignal<u32>, input, action: &'a ReadSignal<fn(&mut DefaultField)>| {
        // derive timer from looping interval
        let timer = delay.map(cx, move |d| Timer::new(cx, *d));

        timer::create_timer_finish_effect(cx, timer, move || {
            let state = inputs.get_untracked().borrow().get_state(&input);
            if state.is_pressed() {
                util::with_signal_mut_untracked(field_signal, |field| action.get()(field));
            }
            state.is_held() // continue the timer loop if the input is held (pressed or suppressed)
        });

        timer
    };

    // timer loop executing an action on an interval after an initial buffer timeout
    let buffered_loop_timer = |delays: &'a ReadSignal<_>, input, action: &'a ReadSignal<fn(&mut DefaultField)>| {
        // derive timers from buffer and loop durations
        let buffer_timer = delays.map(cx, move |(b, _)| Timer::new(cx, *b));
        let loop_timer = loop_timer(delays.map(cx, |d| d.1), input, action);

        timer::create_timer_finish_effect(cx, buffer_timer, move || {
            // apply the action if the input is still held down
            if inputs.get_untracked().borrow().get_state(&input).is_pressed() {
                util::with_signal_mut_untracked(field_signal, |field| action.get()(field));
            }
            loop_timer.get().start(); // activate the loop timer
            false
        });

        buffer_timer
    };

    // looping input timers
    let left_timer = buffered_loop_timer(das_arr, Input::Left, left_action);
    let right_timer = buffered_loop_timer(das_arr, Input::Right, right_action);
    let soft_drop_timer = loop_timer(sdr, Input::SoftDrop, soft_drop_action);

    let last_line_clear = create_signal(cx, None::<LineClear>);
    let topped_out = create_selector(cx, || field_signal.get().borrow().topped_out());

    // gravity timer
    let gravity_delay = util::create_config_selector(cx, config, |c| c.gravity_delay);
    let gravity_action = loop_timer_shift_action!(1, 0, gravity_delay);
    let gravity_timer = gravity_delay.map(cx, move |d| {
        let timer = Timer::new(cx, *d);
        timer.start();
        timer
    });
    timer::create_timer_finish_effect(cx, gravity_timer, || {
        if config.get_untracked().borrow().gravity_enabled {
            util::with_signal_mut_untracked(field_signal, |field| gravity_action.get()(field));
        }
        true
    });

    // lock delay timer
    let lock_delay = util::create_config_selector(cx, config, |c| c.lock_delay);
    let lock_delay_timer = lock_delay.map(cx, move |d| Timer::new(cx, *d));
    let cur_piece = create_selector(cx, || field_signal.get().borrow().cur_piece().coords().clone());
    let lock_delay_piece = create_signal(cx, (*cur_piece.get()).clone());

    // auto lock
    timer::create_timer_finish_effect(cx, lock_delay_timer, || {
        // lock the piece if it is the same as when the timer started
        let still_same_piece = cur_piece.get_untracked() == lock_delay_piece.get_untracked();
        if config.get_untracked().borrow().auto_lock_enabled && still_same_piece {
            hard_drop(field_signal, bag, spin_types, last_line_clear);
        }
        false
    });

    // starts lock delay timer if the current piece touches the stack
    create_effect(cx, || {
        cur_piece.track();
        if field_signal.get_untracked().borrow().cur_piece_cannot_move_down() {
            util::with_signal_mut_untracked(field_signal, |field| field.activate_lock_delay());
            lock_delay_piece.set((*cur_piece.get()).clone());
            lock_delay_timer.get().start();
        }
    });

    // toggle running state of timers
    let run_timers = create_signal(cx, true);
    create_effect(cx, || {
        elapsed_timer.get().stop();
        gravity_timer.get().stop();
        lock_delay_timer.get().stop();

        // set elapsed time accurately
        time_elapsed.set(Date::now() - *start_time.get_untracked());

        if *run_timers.get() {
            time_elapsed.set(0.0);
            start_time.set(Date::now());

            // don't start lock delay timer
            elapsed_timer.get().start();
            gravity_timer.get().start();
        }
    });

    // current game goal
    let goal_type = util::create_config_selector(cx, config, |c| c.goal_type);
    let make_goal = move || match *goal_type.get() {
        GoalTypes::None => goal::none(cx),
        GoalTypes::LinesCleared => goal::lines_cleared(cx, config, last_line_clear),
        GoalTypes::TimeLimit => goal::time_limit(cx, config, time_elapsed),
    };

    // not mapped signal as it must be mutable (for resetting)
    let goal = create_signal(cx, make_goal());
    create_effect(cx, move || goal.set(make_goal()));

    // top out to end the game when the goal is reached or the player topped out naturally
    create_effect(cx, move || {
        if goal.get().is_completed() {
            util::with_signal_mut(field_signal, |field| field.top_out());
        }
        if *topped_out.get() {
            run_timers.set(false);
        }
    });

    let reset_board = move || {
        last_line_clear.set(None);
        goal.set(make_goal());

        let kinds = piece_kinds.get();
        let mut new_bag = SingleBag::new((*kinds).clone());
        let field = DefaultField::new(c.field_width, c.field_height, c.field_hidden, &*kinds, &mut new_bag);

        run_timers.set(true);

        field_signal.set(RefCell::new(field));
        bag.set(RefCell::new(new_bag));
    };

    let ui_enabled = use_context::<Signal<UiEnabled>>(cx);

    let keydown_handler = move |e: Event| {
        let e = e.dyn_into::<KeyboardEvent>().unwrap();
        let c = config.get();
        let c = c.borrow();

        c.keybinds.get_by_right(&e.key()).map(|input| {
            // don't do anything if the input was already pressed
            // these presses come from the operating system repeating inputs automatically
            if util::with_signal_mut(inputs, |inputs| inputs.set_pressed(input)).is_pressed() {
                return;
            }

            // actions possible after topping out
            match input {
                Input::Reset => reset_board(),
                Input::ShowHideUi => ui_enabled.set((!**ui_enabled.get()).into()),
                _ => {}
            }

            if *topped_out.get() && c.topping_out_enabled {
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
                    Input::RotateCw => drop(field.try_rotate_cw(c.kick_table.table())),
                    Input::RotateCcw => drop(field.try_rotate_ccw(c.kick_table.table())),
                    Input::Rotate180 => drop(field.try_rotate_180(c.kick_table_180.table())),
                    Input::SwapHold => util::with_signal_mut_silent(bag, |bag| field.swap_hold_piece(bag)),
                    _ => {}
                }
            });

            // see comment below
            if *input == Input::HardDrop {
                hard_drop(field_signal, bag, spin_types, last_line_clear);
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

        c.keybinds.get_by_right(&e.key()).map(|input| {
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

    let move_limit = util::create_config_selector(cx, config, |c| c.move_limit);
    let actions_since_lock_delay = create_selector(cx, || {
        field_signal.get().borrow().actions_since_lock_delay().unwrap_or(0)
    });

    // action limit (after piece touches stack)
    create_effect(cx, || {
        let limit_reached = actions_since_lock_delay.get() == move_limit.get_untracked();
        if config.get_untracked().borrow().move_limit_enabled && limit_reached {
            hard_drop(field_signal, bag, spin_types, last_line_clear);
        }
    });

    let style_values = util::create_config_selector(cx, config, |c| (c.field_zoom * 100.0, c.vertical_offset));
    let game_style = style_values.map(cx, |d| format!("transform: scale({}%); margin-top: {}px;", d.0, d.1));

    view! { cx,
        div(class="game", tabindex="0", style=game_style.get(), on:keydown=keydown_handler, on:keyup=keyup_handler) {
            div(class="field-panel") {
                div(class="hold-piece") { HoldPiece {} }
                div(class="game-stats") { Stats { last_line_clear, goal } }
            }
            div(class="field") { Field {} }
            div(class="next-queue") { NextQueue { bag } }
        }
    }
}

pub type AssetCache = HashMap<String, HtmlImageElement>;

fn make_asset_cache() -> AssetCache {
    <TetrominoSrs as PieceKindTrait>::iter()
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

fn hard_drop(
    field: &Signal<RefCell<DefaultField>>,
    bag: &Signal<RefCell<impl Randomizer>>,
    spin_types: &ReadSignal<SpinTypes>,
    last_line_clear: &Signal<Option<LineClear>>,
) {
    util::with_signal_mut_untracked(field, |field| {
        util::with_signal_mut_silent_untracked(bag, |bag| {
            // silent so effects depending on this don't try to double borrow the field
            last_line_clear.set_silent(Some(field.hard_drop(bag, spin_types.get().detector())))
        })
    });
    util::notify_subscribers(last_line_clear);
    util::notify_subscribers(bag);
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
