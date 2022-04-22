use std::cell::RefCell;

use crate::{board::Board, config::Config, util};

use sycamore::{
    component,
    generic_node::Html,
    prelude::{ReadSignal, Scope, use_context, Signal},
    view,
    view::View,
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
pub fn Game<'a, G: Html>(cx: Scope<'a>) -> View<G> {
    let config = use_context::<Signal<RefCell<Config>>>(cx);
    let piece_type = util::create_config_selector(cx, config, |c| c.piece_type);

    view! { cx,
        Router {
            integration: HistoryIntegration::new(),
            view: move |cx, route: &ReadSignal<Routes>| {
                view! { cx,
                    div(class="content") {
                        (match route.get().as_ref() {
                            Routes::Home => view! { cx, Board { piece_type } },
                            Routes::NotFound => view! { cx, p(class="loading-text") { "not found" } }
                        })
                    }
                }
            }
        }
    }
}
