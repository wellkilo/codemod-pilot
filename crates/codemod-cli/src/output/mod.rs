//! Output formatting modules.

pub mod diff;
pub mod interactive;
pub mod report;

pub use diff::DiffPrinter;
pub use interactive::InteractivePrompt;
pub use report::ReportPrinter;
