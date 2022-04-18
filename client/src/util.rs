use std::cell::RefCell;

use sycamore::prelude::Signal;

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
pub fn notify_subscribers<T>(signal: &Signal<RefCell<T>>) {
    let value_rc = signal.get_untracked(); // TODO: will this being untracked cause issues?
    signal.set_rc(value_rc);
}
