#![deny(unsafe_op_in_unsafe_fn)]

pub mod alloc;
mod gc_heap;
pub mod graph;
mod handle_manager;
mod handle_store;
pub mod object;
pub mod platform;
pub mod runtime;
mod segment;
mod suspend;
mod suspension_state;
mod util;
