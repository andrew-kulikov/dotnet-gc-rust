#![deny(unsafe_op_in_unsafe_fn)]

type HRESULT = u32;

const S_OK: HRESULT = 0;
const E_FAIL: HRESULT = 0x80004005;
const E_OUTOFMEMORY: HRESULT = 0x8007000E;

/// Confirms that the native C++ shim can cross the stable C ABI into Rust.
///
/// The loader-boundary milestone deliberately stops after this probe. No Rust
/// panic, allocation policy, or CoreCLR interface layout crosses this boundary.
#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_loader_probe() -> HRESULT {
    println!("rust_gc_loader_probe() called");
    S_OK
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_initialize() -> HRESULT {
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
}
