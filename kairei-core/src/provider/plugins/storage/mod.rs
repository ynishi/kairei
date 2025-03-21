//! Storage backends for persistent shared memory.
//!
//! This module contains implementations of various storage backends
//! for the PersistentSharedMemoryPlugin and SistenceStorageService.

pub mod gcp;
pub mod in_memory;
pub mod local_fs;
pub mod sistence_storage;
#[cfg(test)]
mod sistence_storage_test;
