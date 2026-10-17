//! Domain Layer (pure business logic, zero I/O).

pub mod branch;
pub mod config;
pub mod conventional;
pub mod error;
pub mod port;
pub mod semver;
pub mod slot;

pub use branch::{validate_leaf_name, BranchType};
pub use config::{
    default_worktree_root, expand_home, CacheConfig, CacheStrategy, Config, DockerConfig,
    FilesConfig, HooksConfig, ProjectConfig,
};
pub use conventional::{
    determine_bump, parse_conventional_commit, ConventionalCommit, STANDARD_TYPES,
};
pub use error::{BranchError, ConfigError, PortError};
pub use port::{calculate_effective_ports, format_port_env_key};
pub use semver::{parse_semver, validate_semver, BumpType, SemVer};
pub use slot::allocate_lowest_slot;
