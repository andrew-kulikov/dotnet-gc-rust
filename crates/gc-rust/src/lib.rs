#![deny(unsafe_op_in_unsafe_fn)]

pub mod core;
pub mod platform;
pub mod runtime;

type HResult = u32;

const S_OK: HResult = 0;

/// Confirms that the native C++ shim can cross the stable C ABI into Rust.
///
/// The loader-boundary milestone deliberately stops after this probe. No Rust
/// panic, allocation policy, or CoreCLR interface layout crosses this boundary.
#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_loader_probe() -> HResult {
    println!("rust_gc_loader_probe() called");
    S_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_initialize() -> HResult {
    println!("rust_gc_initialize() called");
    S_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_handle_manager_initialize() -> bool {
    println!("rust_gc_handle_manager_initialize() called");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_probe_succeeds() {
        assert_eq!(rust_gc_loader_probe(), S_OK);
    }

    #[test]
    fn loader_initialize_dummy_ok() {
        assert_eq!(rust_gc_initialize(), S_OK);
    }

    #[test]
    fn handle_manager_initialize_succeeds() {
        assert!(rust_gc_handle_manager_initialize());
    }
}
