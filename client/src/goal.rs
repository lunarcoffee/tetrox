use std::{cell::RefCell, ops::AddAssign};

use sycamore::{
    generic_node::Html,
    prelude::{create_effect, create_selector, create_signal, ReadSignal, Scope, Signal},
    view,
    view::View,
};
use tetrox::field::LineClear;

use crate::{config::Config, util};

// a goal for completion of a game (e.g. clear 40 lines)
pub struct Goal<'a, G: Html>(&'a ReadSignal<bool>, View<G>, bool);

impl<'a, G: Html> Goal<'a, G> {
    pub fn is_completed(&self) -> bool { *self.0.get() }

    pub fn view(&self) -> &View<G> { &self.1 }

    // whether to show the default timer (counting up) when this goal is used
    // usually true unless the goal has a built-in timer
    pub fn show_elapsed_time(&self) -> bool { self.2 }
}

pub fn none<'a, G: Html>(cx: Scope<'a>) -> Goal<'a, G> { Goal(create_signal(cx, false), view! { cx, }, true) }

// goal which completes upon reaching a certain number of lines cleared
pub fn lines_cleared<'a, G: Html>(
    cx: Scope<'a>,
    config: &'a Signal<RefCell<Config>>,
    clear_type: &'a Signal<Option<LineClear>>,
) -> Goal<'a, G> {
    // simple line clear counter
    let new_lines_cleared = clear_type.map(cx, |c| c.as_ref().map(|c| c.n_lines()).unwrap_or(0) as u32);
    let n_cleared = create_signal(cx, 0);
    create_effect(cx, || n_cleared.modify().add_assign(*new_lines_cleared.get()));

    let n_lines = util::create_config_selector(cx, config, |c| c.goal_n_lines);
    let completed = n_lines.map(cx, |n| n <= &n_cleared.get());

    let view = view! { cx,
        p(class="game-stats-label") { "LINES" }
        p(class="game-stats-display", style="direction: ltr;") { (format!("{}/{}", n_cleared.get(), n_lines.get())) }
    };

    Goal(completed, view, true)
}

// goal which completes upon reaching the expiration of a time limit
pub fn time_limit<'a, G: Html>(
    cx: Scope<'a>,
    config: &'a Signal<RefCell<Config>>,
    time_elapsed: &'a Signal<f64>,
) -> Goal<'a, G> {
    // remaining time
    let limit_millis = util::create_config_selector(cx, config, |c| c.goal_time_limit_secs * 1_000);
    let time_remaining = time_elapsed.map(cx, |t| (*limit_millis.get() as f64) - t);
    let completed = create_selector(cx, || *time_remaining.get() <= 0.0);

    let view = view! { cx,
        p(class="game-stats-label") { "TIME LEFT" }
        p(class="game-stats-display", style="direction: ltr;") { (util::format_duration(*time_remaining.get())) }
    };

    Goal(completed, view, false)
}
