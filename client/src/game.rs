use std::{
    cell::RefCell,
    ops::{AddAssign, Deref},
    time::Duration,
};

use crate::{
    board::Board,
    config::Config,
    timer::{self, Timer},
    util,
};

use js_sys::Date;
use sycamore::{
    component,
    generic_node::Html,
    prelude::{
        create_effect, create_signal, provide_context, provide_context_ref, use_context, ReadSignal, Scope, Signal,
    },
    view,
    view::View,
};
use sycamore_router::{HistoryIntegration, Route, Router};
use tetrox::field::LineClear;

#[derive(Route)]
pub enum Routes {
    #[to("/")]
    Home,
    #[not_found]
    NotFound,
}

#[component]
pub fn Game<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let config = use_context::<Signal<RefCell<Config>>>(cx);
    let piece_type = util::create_config_selector(cx, config, |c| c.piece_type);

    let time_elapsed = create_signal(cx, 0.0);
    provide_context_ref(cx, time_elapsed);

    // measuring time elapsed since last board reset
    let start_time = create_signal(cx, Date::now());
    let elapsed_timer = create_signal(cx, Timer::new(cx, 33)); // TODO: make timer accuracy configurable
    timer::create_timer_finish_effect(cx, elapsed_timer, move || {
        time_elapsed.set(Date::now() - *start_time.get());
        true
    });

    // toggle running state of timers
    let run_timers = create_signal(cx, TimersPaused(true));
    create_effect(cx, || {
        elapsed_timer.get().stop();
        time_elapsed.set(Date::now() - *start_time.get_untracked());

        if **run_timers.get() {
            time_elapsed.set(0.0);
            start_time.set(Date::now());
            elapsed_timer.get().start();
        }
    });

    view! { cx,
        Router {
            integration: HistoryIntegration::new(),
            view: move |cx, route: &ReadSignal<Routes>| {
                view! { cx,
                    div(class="content") {
                        (match route.get().as_ref() {
                            Routes::Home => view! { cx, Board { piece_type, run_timers } },
                            Routes::NotFound => view! { cx, p(class="loading-text") { "not found" } }
                        })
                    }
                }
            }
        }
    }
}

pub struct TimersPaused(bool); // TODO: extract newtype definition for context signals to macro?

impl Deref for TimersPaused {
    type Target = bool;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl From<bool> for TimersPaused {
    fn from(b: bool) -> Self { TimersPaused(b) }
}
