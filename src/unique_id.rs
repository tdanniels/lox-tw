use std::sync::Mutex;

static COUNTER: Mutex<u128> = Mutex::new(0);

pub fn unique_id() -> u128 {
    let mut guard = COUNTER.lock().unwrap();
    *guard += 1;
    *guard
}
