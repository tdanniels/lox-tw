use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};

static COUNTER_USIZE: AtomicUsize = AtomicUsize::new(0);
static COUNTER_U128: Mutex<u128> = Mutex::new(0);

pub fn unique_usize() -> usize {
    COUNTER_USIZE.fetch_add(1, Ordering::Relaxed)
}

pub fn unique_u128() -> u128 {
    let mut guard = COUNTER_U128.lock().unwrap();
    *guard += 1;
    *guard
}
