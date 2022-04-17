use std::cell::RefCell;

use sycamore::prelude::Signal;

// allows `op` to run with a `&mut T` of the signal value
pub fn with_signal_mut<T, R>(signal: &Signal<RefCell<T>>, op: impl Fn(&mut T) -> R) -> R {
    let value_rc = signal.get();
    let result = {
        // this has to be in a new scope so this mutable borrow is dropped before `Signal::set_rc` tries to mutably
        // borrow its inner rc again
        let mut value = value_rc.borrow_mut();
        op(&mut value)
    };
    signal.set_rc(value_rc);
    result
}
