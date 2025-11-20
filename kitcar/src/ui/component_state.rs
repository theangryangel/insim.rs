use std::cell::RefCell;
/// Not-a-signal-signal.
use std::rc::Rc;

/// Component state, used as part of cx.use_state.
/// We can use Rc here rather than Arc and Mutexs because we're assuming that
/// the UI runs on it's own Tokio LocalSet / thread.
/// It allows for shared, mutable access to a value within a single thread.
/// Cloning a is cheap, as it only clones the reference-counted pointer.
#[derive(Debug)]
pub struct ComponentState<T> {
    value: Rc<RefCell<T>>,
}

impl<T> ComponentState<T> {
    /// Creates a new Signal with an initial value.
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    /// Updates the value within the Signal.
    /// This will notify any subscribers in a more advanced implementation.
    pub fn set(&self, new_value: T) {
        *self.value.borrow_mut() = new_value;
    }
}

// Implement `Clone` so the signal can be easily shared.
impl<T> Clone for ComponentState<T> {
    fn clone(&self) -> Self {
        Self {
            value: Rc::clone(&self.value),
        }
    }
}

// Add a convenient `get` method for types that can be copied (like numbers).
impl<T: Copy> ComponentState<T> {
    /// Returns a copy of the value.
    /// This is a convenience for simple types like `i32`, `bool`, etc.
    pub fn get(&self) -> T {
        *self.value.borrow()
    }
}
