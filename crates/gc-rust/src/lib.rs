#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
pub mod core;
mod gc_heap;
mod handle_manager;
mod handle_store;
pub mod object;
pub mod platform;
pub mod runtime;
mod segment;
