use sycamore::{
    component,
    generic_node::Html,
    prelude::{ReadSignal, Scope},
    view,
    view::View,
};
use sycamore_router::{HistoryIntegration, Route, Router};

#[derive(Route)]
enum Routes {
    #[to("/")]
    Home,
    #[not_found]
    NotFound,
}

#[component]
pub fn Game<'a, G: Html>(cx: &'a Scope<'a>) -> View<G> {
    view! { cx,
        Router {
            integration: HistoryIntegration::new(),
            view: |cx, route: &ReadSignal<Routes>| {
                view! { cx,
                    div(class="content") {
                        div(class="bg-gradient")
                        (match route.get().as_ref() {
                            Routes::Home => view! { cx,
                                p(class="loading-text") { "game" }
                            },
                            Routes::NotFound => view! { cx, p(class="loading-text") { "not found" } }
                        })
                    }
                }
            }
        }
    }
}
