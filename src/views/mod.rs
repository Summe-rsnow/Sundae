//! Sundae's 5 primary feature views.

pub mod buckets;
pub mod config;
pub mod discover;
pub mod installed;
pub mod updates;

pub use buckets::buckets_view;
pub use config::config_view;
pub use discover::discover_view;
pub use installed::installed_view;
pub use updates::updates_view;
