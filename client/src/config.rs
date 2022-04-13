use sycamore::{component, generic_node::Html, prelude::Scope, view, view::View};

#[component]
fn SectionHeading<'a, G: Html>(cx: &'a Scope<'a>, section: &'static str) -> View<G> {
    view! { cx, p(class="config-heading") { (section) } }
}

#[component]
pub fn ConfigPanel<'a, G: Html>(cx: &'a Scope<'a>) -> View<G> {
    view! { cx,
        div(class="config-panel") {
            SectionHeading("Gameplay")
        }    
    }
}
