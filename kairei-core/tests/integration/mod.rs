//! Integration tests for the kairei-core crate
//! 
//! These tests verify the behavior of the entire compilation pipeline
//! from source code to AST, focusing on error handling and location tracking.

pub mod span_tracking;
pub mod span_tracking_fixed_v4;
pub mod system_span_tracking;
