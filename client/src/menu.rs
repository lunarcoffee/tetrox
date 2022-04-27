use std::cell::RefCell;

use crate::{
    board::Board,
    config::{Config, GoalTypes},
    util::{self, Padding, SectionHeading},
};

use sycamore::{
    component,
    generic_node::Html,
    motion::Tweened,
    prelude::{create_memo, use_context, ReadSignal, Scope, Signal},
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

#[derive(Prop)]
pub struct MenuProps<'a> {
    ui_offset: &'a Tweened<'a, f64>,
}

#[component]
pub fn Menu<'a, G: Html>(cx: Scope<'a>, props: MenuProps<'a>) -> View<G> {
    let lines_cleared_preset = move |label, n_lines| view! { cx, GoalPresetButton { label, goal_type: GoalTypes::LinesCleared, n_lines, time_limit_secs: 0 } };
    let time_limit_preset = move |label, time_limit_secs| view! { cx, GoalPresetButton { label, goal_type: GoalTypes::TimeLimit, n_lines: 0, time_limit_secs } };

    let menu = view! { cx,
        p(class="logo") { "Tetrox" }

        ModeButton("Singleplayer")
        ModeButton("Scores (not implemented)")
        Padding(1)

        SectionHeading("Sprint")
        div(class="menu-button-box menu-button-box-l") {
            (lines_cleared_preset("20 lines", 20))
            (lines_cleared_preset("40 lines", 40))
            (lines_cleared_preset("100 lines", 100))
            (lines_cleared_preset("1000 lines", 1_000))
        }

        SectionHeading("Ultra")
        div(class="menu-button-box menu-button-box-l") {
            (time_limit_preset("1 minute", 60))
            (time_limit_preset("2 minutes", 120))
            (time_limit_preset("5 minutes", 300))
            (time_limit_preset("1 hour", 3_600))
        }
    };

    let ui_offset = props.ui_offset;
    let menu_style = create_memo(cx, || format!("margin-left: calc(-{}rem + 20px);", ui_offset.get()));

    view! { cx,
        Router {
            integration: HistoryIntegration::new(),
            view: move |cx, route: &ReadSignal<Routes>| {
                view! { cx,
                    div(class="content") {
                        div(class="menu", style=menu_style.get()) { (menu) }
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
fn ModeButton<'a, G: Html>(cx: Scope<'a>, label: &'static str) -> View<G> {
    view! { cx,
        div(class="mode-button") {
            p(class="mode-button-text") { (label) }
        }
    }
}

#[derive(Prop)]
struct GoalPresetProps {
    label: &'static str,

    goal_type: GoalTypes,
    n_lines: u32,
    time_limit_secs: u64,
}

#[component]
fn GoalPresetButton<'a, G: Html>(cx: Scope<'a>, props: GoalPresetProps) -> View<G> {
    let config = use_context::<Signal<RefCell<Config>>>(cx);

    view! { cx,
        div(class="menu-option menu-option-l") {
            input(
                type="button",
                value=props.label,
                on:click=move |_| util::with_signal_mut(config, |c| {
                    c.goal_type = props.goal_type;
                    c.goal_n_lines = props.n_lines;
                    c.goal_time_limit_secs = props.time_limit_secs;
                }),
            )
        }
    }
}
