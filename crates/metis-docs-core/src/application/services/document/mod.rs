pub mod discovery;
pub mod validation;
pub mod creation;
pub mod deletion;

pub use discovery::DocumentDiscoveryService;
pub use validation::DocumentValidationService;
pub use creation::DocumentCreationService;
pub use deletion::{DeletionService, DeletionResult};