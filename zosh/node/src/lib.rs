//! Zorch Node Implementation

pub mod cmd;
pub mod dev;
pub mod rpc;
pub mod storage;

/// Git commit hash embedded at build time
pub const GIT_HASH: &str = env!("GIT_HASH");

/// Git branch embedded at build time
pub const GIT_BRANCH: &str = env!("GIT_BRANCH");

/// Build version string (for use with clap)
pub const VERSION: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_HASH"),
    " ",
    env!("GIT_BRANCH"),
    ")"
);
