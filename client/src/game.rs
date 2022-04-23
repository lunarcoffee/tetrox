use crate::board::Board;

use sycamore::{
    component,
    generic_node::Html,
    prelude::{ReadSignal, Scope},
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
    view! { cx,
        Router {
            integration: HistoryIntegration::new(),
            view: move |cx, route: &ReadSignal<Routes>| {
                view! { cx,
                    div(class="content") {
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
