// Module organization for the StatelessRelevantMemory implementation

// Utility functions
mod utility_functions;

// Core implementation
mod core;

// Core operations
mod core_operations;

// Graph operations
mod graph_operations;

// Search operations
mod search_operations;

// Link operations
mod link_operations;

// Context operations
mod context_operations;

// Metadata operations
mod metadata_operations;

// Memory processing operations
mod memory_processing;

// Re-export the StatelessRelevantMemory struct
pub use core::StatelessRelevantMemory;

// Re-export utility functions
pub use utility_functions::*;
