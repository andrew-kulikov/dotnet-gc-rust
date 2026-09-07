type HResult = u32;

const S_OK: HResult = 0;

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
