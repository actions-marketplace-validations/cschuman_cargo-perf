// Positive: a synchronous `std::sync::Mutex` guard is held while a `match` arm
// *guard expression* (`0 if is_ready().await`) awaits. The guard expression is
// evaluated before any arm body runs, so the await happens with the std guard
// live — a genuine deadlock risk. The analyzer must check arm guards, not only
// the scrutinee and the arm bodies.
use std::sync::Mutex;

async fn route(m: &Mutex<i32>, ev: u8) {
    let guard = m.lock().unwrap();
    let _v = *guard;
    match ev {
        0 if is_ready().await => {} // perf-expect: lock-across-await
        _ => {}
    }
}

async fn is_ready() -> bool {
    true
}
