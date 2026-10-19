use crate::{
    runtime::{SuspendReason, gc_to_clr},
    util::log_info,
};

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_suspend_ee() {
    log_info!("rust_gc_suspend_ee()");
    // SAFETY: the native caller must initiate suspension on an eligible thread.
    unsafe { gc_to_clr().suspend_ee(SuspendReason::SuspendForGc) };
    for i in 0..10 {
        log_info!("rust_gc_suspend_ee() sleeping iteration {}", i);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    log_info!("rust_gc_suspend_ee() finished sleeping");
    unsafe { gc_to_clr().restart_ee(false) };
    log_info!("rust_gc_suspend_ee() completed");
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn rust_gc_restart_ee() {
    log_info!("rust_gc_restart_ee()");
    // SAFETY: the native caller must own the matching suspension on this thread.
    // ZeroGC has not completed a collection during this stopped window.
    unsafe { gc_to_clr().restart_ee(false) };
}
