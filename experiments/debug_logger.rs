use std::sync::atomic::{AtomicU32, Ordering};

static FRAME_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn debug_log(component: &str, message: &str) {
    let frame = FRAME_COUNTER.load(Ordering::Relaxed);
    println!("[DEBUG][Frame {}][{}] {}", frame, component, message);
}

pub fn debug_log_with_data<T: std::fmt::Debug>(component: &str, message: &str, data: &T) {
    let frame = FRAME_COUNTER.load(Ordering::Relaxed);
    println!("[DEBUG][Frame {}][{}] {} | Data: {:?}", frame, component, message, data);
}

pub fn increment_frame() {
    FRAME_COUNTER.fetch_add(1, Ordering::Relaxed);
}

pub fn get_frame_count() -> u32 {
    FRAME_COUNTER.load(Ordering::Relaxed)
}

pub fn reset_frame_count() {
    FRAME_COUNTER.store(0, Ordering::Relaxed);
}