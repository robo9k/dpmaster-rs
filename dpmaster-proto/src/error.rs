//! crate error types

use thiserror::Error;

/// Empty value error
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("is empty")]
pub struct EmptyError;

/// Invalid byte error
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("contains invalid byte {} at {:x}", .1[*.0], .0)]
pub struct InvalidByteError(pub(crate) usize, pub(crate) Vec<u8>);

/// Errors for [Challenge](crate::messages::Challenge)
#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum InvalidChallengeError {
    #[error(transparent)]
    Empty(#[from] EmptyError),
    #[error(transparent)]
    InvalidByte(#[from] InvalidByteError),
}

/// Possible crate errors
#[derive(Error, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    /// Invalid [`crate::Challenge`]
    #[error(transparent)]
    InvalidChallenge(#[from] InvalidChallengeError),

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
    Dpmaster(I, crate::deserializer::ErrorKind),
}

impl<I> crate::deserializer::ParseError<I> for DeserializationError<I> {
    fn from_dpmaster_error_kind(input: I, kind: crate::deserializer::ErrorKind) -> Self {
        Self::Dpmaster(input, kind)
    }

    fn append_dpmaster(_input: I, _kind: crate::deserializer::ErrorKind, other: Self) -> Self {
        other
    }
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
