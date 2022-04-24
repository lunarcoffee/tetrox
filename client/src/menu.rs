use std::cell::RefCell;

use crate::{
    board::Board,
    config::{Config, GoalTypes},
    util,
};

use sycamore::{
    component,
    generic_node::Html,
    prelude::{use_context, ReadSignal, Scope, Signal},
    view,
    view::View,
    Prop,
};
use sycamore_router::{HistoryIntegration, Route, Router};

#[derive(Route)]
pub enum Routes {
    #[to("/")]
    Home,
    #[not_found]
    NotFound,
}

#[component]
pub fn Menu<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let lines_cleared_preset = move |label, n_lines| view! { cx, 
        GoalPreset { label, goal_type: GoalTypes::LinesCleared, n_lines, time_limit_secs: 0 } 
    };
    let time_limit_preset = move |label, time_limit_secs| view! { cx, 
        GoalPreset { label, goal_type: GoalTypes::TimeLimit, n_lines: 0, time_limit_secs } 
    };

    view! { cx,
        Router {
            integration: HistoryIntegration::new(),
            view: move |cx, route: &ReadSignal<Routes>| {
                view! { cx,
                    div(class="content") {
                        div(class="menu") {
                            p(class="logo") { "Tetrox" }

                            PresetListHeading("Sprint")
                            (lines_cleared_preset("20 lines", 20))
                            (lines_cleared_preset("40 lines", 40))
                            (lines_cleared_preset("100 lines", 100))
                            (lines_cleared_preset("1000 lines", 1_000))

                            PresetListHeading("Ultra")
                            (time_limit_preset("1 minute", 60))
                            (time_limit_preset("2 minutes", 120))
                            (time_limit_preset("5 minutes", 300))
                        }
                        (match route.get().as_ref() {
                            Routes::Home => view! { cx, Board {} },
                            Routes::NotFound => view! { cx, p(class="loading-text") { "not found" } }
                        })
                    }
                }
            }
        }
    }
}

#[component]
fn PresetListHeading<'a, G: Html>(cx: Scope<'a>, label: &'a str) -> View<G> {
    view! { cx, p(class="preset-list-heading") { (label.to_uppercase().to_string()) } }
}

#[derive(Prop)]
struct GoalPresetProps {
    label: &'static str,

    goal_type: GoalTypes,
    n_lines: u32,
    time_limit_secs: u64,
}

#[component]
fn GoalPreset<'a, G: Html>(cx: Scope<'a>, props: GoalPresetProps) -> View<G> {
    let config = use_context::<Signal<RefCell<Config>>>(cx);

    view! { cx,
        div(
            class="preset-button",
            on:click=move |_| util::with_signal_mut(config, |c| {
                c.goal_type = props.goal_type;
                c.goal_n_lines = props.n_lines;
                c.goal_time_limit_secs = props.time_limit_secs;
            })
        ) {
            p(class="preset-button-text") { (props.label) }
        }
    }
}
