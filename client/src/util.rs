use std::{cell::RefCell, time::Duration};

use sycamore::{
    component,
    generic_node::Html,
    prelude::{create_selector, ReadSignal, Scope, Signal},
    view,
    view::View,
};

use crate::config::Config;

// allows `op` to run with a `&mut T` of the signal value
// not notifying subscribers is sometimes necessary to avoid dependency issues in nested calls
// getting untracked is sometimes necessary to avoid circular updates
pub fn with_signal_mut_silent_untracked<T, R>(signal: &Signal<RefCell<T>>, mut op: impl FnMut(&mut T) -> R) -> R {
    let value_rc = signal.get_untracked();
    let result = {
        // this has to be in a new scope so this mutable borrow is dropped before `Signal::set_rc` tries to mutably
        // borrow its inner rc again
        let mut value = value_rc.borrow_mut();
        op(&mut value)
    };
    signal.set_rc_silent(value_rc);
    result
}

pub fn with_signal_mut_untracked<T, R>(signal: &Signal<RefCell<T>>, op: impl FnMut(&mut T) -> R) -> R {
    let value = with_signal_mut_silent_untracked(signal, op);
    notify_subscribers(signal);
    value
}

pub fn with_signal_mut_silent<T, R>(signal: &Signal<RefCell<T>>, op: impl FnMut(&mut T) -> R) -> R {
    signal.track();
    with_signal_mut_silent_untracked(signal, op)
}

pub fn with_signal_mut<T, R>(signal: &Signal<RefCell<T>>, op: impl FnMut(&mut T) -> R) -> R {
    let value = with_signal_mut_silent(signal, op);
    notify_subscribers(signal);
    value
}

// not sure why this function is no longer public api like in 0.7.x but oh well
pub fn notify_subscribers<T>(signal: &Signal<T>) {
    let value_rc = signal.get_untracked();
    signal.set_rc(value_rc);
}

// used to select specific config options to update on as opposed to updating on every config value change, even if the
// updated value isn't used in a given computation
pub fn create_config_selector<'a, T, F>(
    cx: Scope<'a>,
    config: &'a Signal<RefCell<Config>>,
    mut op: F,
) -> &'a ReadSignal<T>
where
    T: PartialEq + 'a,
    F: FnMut(&Config) -> T + 'a,
{
    create_selector(cx, move || op(&config.get().borrow()))
}

pub fn format_duration(millis: f64) -> String {
    let time = Duration::from_millis(millis as u64);

    let millis = time.as_millis() % 1_000;
    let secs = time.as_secs() % 60;
    let mins = time.as_secs() / 60;

    format!("{}:{:02}.{:03}", mins, secs, millis)
}

// vertical padding
#[component]
pub fn Padding<'a, G: Html>(cx: Scope<'a>, px: usize) -> View<G> {
    view! { cx, div(style=format!("min-height: {}px;", px)) }
}

// menu/config section heading
#[component]
pub fn SectionHeading<'a, G: Html>(cx: Scope<'a>, section: &'static str) -> View<G> {
    view! { cx, p(class="section-heading") { (section.to_uppercase()) } }
}
