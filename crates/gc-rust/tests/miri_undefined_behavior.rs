//! An intentionally incorrect test used to demonstrate what Miri detects.
//!
//! This test is ignored by default. Run it only through the documented Miri
//! command; forcing it to run with the native test runner stops before the
//! undefined operation.

#[test]
#[ignore = "intentional undefined behavior demonstration; run only through Miri"]
#[allow(
    clippy::assertions_on_constants,
    reason = "the cfg guard prevents accidental native execution of the UB demo"
)]
fn miri_detects_use_after_free() {
    assert!(
        cfg!(miri),
        "this intentionally invalid test must be executed through Miri"
    );

    let dangling = {
        let allocation = Box::new(42_u64);
        let pointer = Box::into_raw(allocation);

        // SAFETY: This reconstructs the Box exactly once from its original
        // pointer, so freeing the allocation here is valid. Keeping `pointer`
        // afterward is deliberate: it becomes dangling for the Miri demo.
        unsafe {
            drop(Box::from_raw(pointer));
        }

        pointer
    };

    // SAFETY: Intentionally false. The allocation has already been freed.
    // Rust accepts this promise at compile time; Miri detects the use-after-free
    // when it interprets this line.
    let value = unsafe { dangling.read() };

    assert_eq!(value, 42);
}
