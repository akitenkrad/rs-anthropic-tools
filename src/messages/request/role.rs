//! Role types for conversation participants.
//!
//! This module provides the [`Role`] enum for identifying message authors:
//!
//! - `User` - Messages from the user
//! - `Assistant` - Messages from Claude
//!
//! # Example
//!
//! ```rust
//! use anthropic_tools::messages::request::role::Role;
//!
//! let user_role = Role::User;
//! let assistant_role = Role::Assistant;
//!
//! assert_eq!(format!("{}", user_role), "user");
//! assert_eq!(format!("{}", assistant_role), "assistant");
//! ```

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// Role in a conversation (user or assistant)
#[derive(Serialize, Deserialize, Debug, Clone, Display, EnumString, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    #[strum(serialize = "user")]
    User,
    #[strum(serialize = "assistant")]
    Assistant,
}
