//! Licensing: what the engine is running on, and how to ask for it without a
//! dialog.
//!
//! This matters to a headless host for one reason. Asking for a license that
//! cannot be granted makes the engine put up a window offering to evaluate,
//! activate or buy, and that window waits for a person. On a station with
//! nobody at it the process stops there. Passing
//! [`AcquireLicenseOptions::SUPPRESS_STARTUP_DIALOG`] turns that same situation
//! into a returned error, which a host can log, report to its clients and exit
//! on.
//!
//! Reading [`Engine::license_type`](crate::Engine::license_type) needs no
//! acquisition and raises nothing, so a host can check what it is running on
//! before it commits to anything.

mod application_license;
mod held;
mod license_type;
mod options;

pub use application_license::ApplicationLicense;
pub use held::HeldLicense;
pub use license_type::LicenseType;
pub use options::AcquireLicenseOptions;
