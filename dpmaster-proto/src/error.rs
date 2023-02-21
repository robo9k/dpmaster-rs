//! crate error types

use thiserror::Error;

/// Possible crate errors
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    /// Invalid [`crate::Challenge`]
    #[error("Invalid challenge ({byte} at {offset})")]
    InvalidChallenge { byte: u8, offset: usize },
    /// Invalid [`crate::GameName`]
    #[error("Invalid game name ({byte} at {offset})")]
    InvalidGameName { byte: u8, offset: usize },
    /// Invalid [`crate::GameType`]
    #[error("Invalid gametype ({byte} at {offset})")]
    InvalidGameType { byte: u8, offset: usize },
    /// Invalid end of transmission
    ///
    /// In [`crate::GetServersResponseMessage`] or [`crate::GetServersExtResponseMessage`]
    #[error("Invalid EOT (no servers)")]
    InvalidEndOfTransmission,
}

#[derive(Debug, PartialEq)]
pub enum DeserializationError<I> {
    Nom(I, nom::error::ErrorKind),
}

impl<I> nom::error::ParseError<I> for DeserializationError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self::Nom(input, kind)
    }

    fn append(_input: I, _kind: nom::error::ErrorKind, other: Self) -> Self {
        other
    }
}

impl<I> nom::error::ContextError<I> for DeserializationError<I> {
    fn add_context(_input: I, _ctx: &'static str, other: Self) -> Self {
        other
    }
}

use nom::error::ParseError;

impl<I, E> nom::error::FromExternalError<I, E> for DeserializationError<I> {
    fn from_external_error(input: I, kind: nom::error::ErrorKind, _err: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}
