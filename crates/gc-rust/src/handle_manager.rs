#[unsafe(no_mangle)]
pub extern "C" fn rust_gc_handle_manager_initialize() -> bool {
    println!("rust_gc_handle_manager_initialize() called");
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_manager_initialize_succeeds() {
        assert!(rust_gc_handle_manager_initialize());
    }
}
