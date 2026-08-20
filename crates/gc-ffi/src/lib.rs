#![deny(unsafe_op_in_unsafe_fn)]

/// Confirms that the native C++ shim can cross the stable C ABI into Rust.
///
/// The loader-boundary milestone deliberately stops after this probe. No Rust
/// panic, allocation policy, or CoreCLR interface layout crosses this boundary.
#[unsafe(no_mangle)]
pub extern "C" fn gc_rust_loader_probe() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loader_probe_succeeds() {
        assert_eq!(gc_rust_loader_probe(), 0);
    }
}
