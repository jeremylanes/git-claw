//! Domain Layer (pure business logic, zero I/O).

pub mod branch;
pub mod config;
pub mod error;
pub mod port;
pub mod semver;
pub mod slot;

pub use branch::{validate_leaf_name, BranchType};
pub use config::{CacheConfig, CacheStrategy, Config, HooksConfig, ProjectConfig};
pub use error::{BranchError, ConfigError, PortError};
pub use port::{calculate_effective_ports, format_port_env_key};
pub use semver::validate_semver;
pub use slot::allocate_lowest_slot;
