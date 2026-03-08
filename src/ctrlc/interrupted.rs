use std::process;
use std::sync::atomic::{AtomicBool, Ordering};

/// A global variable telling whether the application has been interrupted.
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// Tells whether the program has been interrupted
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Clears the interrupted flag
pub fn unset_interrupted() {
    debug_assert!(INTERRUPTED.load(Ordering::SeqCst));
    INTERRUPTED.store(false, Ordering::SeqCst);
}

pub fn set_interrupted() {
    // If already interrupted (second Ctrl+C), exit immediately
    if INTERRUPTED.swap(true, Ordering::SeqCst) {
        // Exit with 130 (128 + SIGINT signal number 2)
        process::exit(130);
    }
}
