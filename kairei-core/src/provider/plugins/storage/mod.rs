//! Storage backends for persistent shared memory.
//!
//! This module contains implementations of various storage backends
//! for the PersistentSharedMemoryPlugin.

pub mod gcp;
pub mod in_memory;
pub mod local_fs;
