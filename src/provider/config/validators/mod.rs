//! Validators for provider configurations.
//!
//! This module provides validators for provider configurations,
//! including type checker and evaluator validators.

mod evaluator;
mod type_checker;

pub use evaluator::EvaluatorValidator;
pub use type_checker::TypeCheckerValidator;

/// Creates a type checker validator.
pub fn create_type_checker_validator() -> TypeCheckerValidator {
    TypeCheckerValidator::default()
}

/// Creates an evaluator validator.
pub fn create_evaluator_validator() -> EvaluatorValidator {
    EvaluatorValidator::default()
}
