//! crate error types

use thiserror::Error;

/// Possible crate errors
#[derive(Error, Debug)]
pub enum ProtocolError {
    /// Invalid [`crate::GameName`]
    #[error("Invalid game name ({byte} at {offset})")]
    InvalidGameName { byte: u8, offset: usize },
    /// Invalid [`crate::Gametype`]
    #[error("Invalid gametype ({byte} at {offset})")]
    InvalidGametype { byte: u8, offset: usize },
    /// Invalid end of transmission
    ///
    /// In [`crate::GetServersResponseMessage`] or [`crate::GetServersExtResponseMessage`]
    #[error("Invalid EOT (no servers)")]
    InvalidEndOfTransmission,
}
