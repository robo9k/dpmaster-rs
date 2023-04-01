//! Protocol datagram "messages" and related types
//!
//! The dpmaster protocol consists of a few messages that are passed between game servers and the master server to register a game server:
//! - [`heartbeat`](HeartbeatMessage)
//! - [`getinfo`](GetInfoMessage)
//! - [`infoResponse`](InfoResponseMessage)
//!
//! Then there are message that are passed between game clients and the master server to query game servers:
//! - [`getservers`](GetServersMessage)
//! - [`getserversResponse`](GetServersResponseMessage)
//!
//! To support [IPv6](https://en.wikipedia.org/wiki/IPv6) there are extended versions of the previous messages:
//! - [`getserversExt`](GetServersExtMessage)
//! - [`getserversExtResponse`](GetServersExtResponseMessage)

use crate::error::{EmptyError, InvalidByteError, InvalidChallengeError};
use crate::{ProtocolError, Result};

use memchr::memchr2;

fn is_ascii_printable(chr: u8) -> bool {
    chr >= 33 && chr <= 126
}

/// Challenge to authenticate messages
///
/// The dpmaster protocol uses [UDP](https://en.wikipedia.org/wiki/User_Datagram_Protocol) which is spoofable so, to authenticate datagrams and prevent denial-of-service in the [`heartbeat`](HeartbeatMessage) → [`getinfo`](GetInfoMessage) → [`infoResponse`](InfoResponseMessage) chain,
/// a "password" is used that should only be known to the game server and the master server.
#[derive(Debug, PartialEq, Eq)]
pub struct Challenge(Vec<u8>);

impl Challenge {
    /// Creates a new challenge from a container of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let challenge = dpmaster_proto::Challenge::new(*b"A_ch4Lleng3")?;
    /// # Ok::<(), dpmaster_proto::ProtocolError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return an [EmptyError](crate::error::EmptyError) if the supplied bytes are empty.
    /// ```rust
    /// use dpmaster_proto::{error::InvalidChallengeError, messages::Challenge};
    ///
    /// assert!(matches!(Challenge::new(*b"").unwrap_err(), InvalidChallengeError::Empty(..)));
    /// ```
    ///
    /// Will return [InvalidByteError](crate::error::InvalidByteError) if a supplied byte is not [ASCII](https://en.wikipedia.org/wiki/ASCII) printable (code 33 to 126)
    /// or is one of the disallowed characters `\`, `/`, `;`, `"` or `%`.
    /// ```rust
    /// use dpmaster_proto::{error::InvalidChallengeError, messages::Challenge};
    ///
    /// assert!(matches!(Challenge::new(*b"\xFF").unwrap_err(), InvalidChallengeError::InvalidByte(..)));
    /// assert!(matches!(Challenge::new(*b"uhoh;").unwrap_err(), InvalidChallengeError::InvalidByte(..)));
    /// ```
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self, InvalidChallengeError> {
        let bytes = t.into();

        if bytes.is_empty() {
            return Err(EmptyError)?;
        }

        for (offset, byte) in bytes.iter().copied().enumerate() {
            if !is_ascii_printable(byte) || [b'\\', b'/', b';', b'"', b'%'].contains(&byte) {
                return Err(InvalidByteError(offset, bytes))?;
            }
        }

        Ok(Self(bytes))
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for Challenge {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

/// `getinfo` message
#[derive(Debug, PartialEq, Eq)]
pub struct GetInfoMessage {
    challenge: Challenge,
}

impl GetInfoMessage {
    pub fn new(challenge: Challenge) -> Self {
        Self { challenge }
    }

    pub fn challenge(&self) -> &Challenge {
        &self.challenge
    }
}

pub type MaxClientsNumber = std::num::NonZeroU32;

pub type ClientsNumber = u32;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct InfoKey(Vec<u8>);

impl InfoKey {
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self> {
        let bytes = t.into();

        Ok(Self(bytes))
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for InfoKey {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoValue(Vec<u8>);

impl InfoValue {
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self> {
        let bytes = t.into();

        Ok(Self(bytes))
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for InfoValue {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Info(indexmap::IndexMap<InfoKey, InfoValue>);

impl Info {
    pub fn new() -> Self {
        Self(indexmap::IndexMap::new())
    }

    pub fn insert(&mut self, key: InfoKey, value: InfoValue) {
        self.0.insert(key, value);
    }

    pub fn iter(&self) -> indexmap::map::Iter<'_, InfoKey, InfoValue> {
        self.0.iter()
    }

    pub fn challenge(&self) -> &Challenge {
        todo!();
    }

    pub fn sv_maxclients(&self) -> MaxClientsNumber {
        todo!();
    }

    pub fn protocol(&self) -> ProtocolNumber {
        todo!();
    }

    pub fn clients(&self) -> ClientsNumber {
        todo!();
    }

    pub fn gamename(&self) -> Option<&GameName> {
        todo!();
    }

    pub fn gametype(&self) -> Option<&GameType> {
        todo!();
    }
}

/// `infoResponse` message
#[derive(Debug, PartialEq, Eq)]
pub struct InfoResponseMessage {
    info: Info,
}

impl InfoResponseMessage {
    pub fn new(info: Info) -> Self {
        Self { info }
    }

    pub fn info(&self) -> &Info {
        &self.info
    }
}

// protocol name
#[derive(Debug, PartialEq, Eq)]
pub struct ProtocolName(Vec<u8>);

impl ProtocolName {
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self> {
        let bytes = t.into();
        Ok(Self(bytes))
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for ProtocolName {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

impl std::default::Default for ProtocolName {
    fn default() -> Self {
        Self::new(b"DarkPlaces".to_vec()).expect("known value to be valid")
    }
}

/// `heartbeat` message
#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatMessage {
    protocol_name: ProtocolName,
}

impl HeartbeatMessage {
    pub fn new(protocol_name: ProtocolName) -> Self {
        Self { protocol_name }
    }

    pub fn protocol_name(&self) -> &ProtocolName {
        &self.protocol_name
    }
}

/// protocol number
pub type ProtocolNumber = u32;

/// game name
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GameName(Vec<u8>);

impl GameName {
    /// Creates a new game name from a container of bytes.
    ///
    /// Game names can contain neither null bytes nor whitespace.
    ///
    /// # Examples
    /// ```
    /// use dpmaster_proto::GameName;
    /// let game_name = GameName::new(b"Nexuiz".to_vec());
    /// assert!(game_name.is_ok());
    /// ```
    ///
    /// # Errors
    /// This function will return an error if the supplied bytes contain a
    /// null/`0` byte or whitespace/`' '`.
    /// The [`ProtocolError::InvalidGameName`] error will include the invalid byte
    /// as well as the first offset it occurred at.
    /// ```
    /// use dpmaster_proto::{GameName, ProtocolError};
    /// let game_name = GameName::new(b"invalid example".to_vec());
    /// assert_eq!(game_name, Err(ProtocolError::InvalidGameName {byte: b' ', offset: 7}));
    /// ```
    // FIXME: Comparing private fields in a public doctest feels wrong. Maybe compare debug output instead?
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self> {
        let bytes = t.into();
        match memchr2(b'\0', b' ', &bytes) {
            Some(i) => Err(ProtocolError::InvalidGameName {
                offset: i,
                byte: bytes[i],
            }),
            None => Ok(Self(bytes)),
        }
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for GameName {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

impl std::str::FromStr for GameName {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::new(s.as_bytes().to_vec())?)
    }
}

/// game type
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GameType(Vec<u8>);

impl GameType {
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self> {
        let bytes = t.into();
        Ok(Self(bytes))
    }
}

impl<I: std::slice::SliceIndex<[u8]>> std::ops::Index<I> for GameType {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        std::ops::Index::index(&self.0, index)
    }
}

impl std::str::FromStr for GameType {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self::new(s.as_bytes().to_vec())?)
    }
}

/// filter options for `getservers`
#[derive(Debug, PartialEq, Eq)]
pub struct FilterOptions {
    /// `gametype=X` filter option
    gametype: Option<GameType>,
    /// empty servers option
    empty: bool,
    /// full servers option
    full: bool,
}

impl FilterOptions {
    pub fn new(gametype: Option<GameType>, empty: bool, full: bool) -> Self {
        Self {
            gametype,
            empty,
            full,
        }
    }

    pub fn gametype(&self) -> Option<&GameType> {
        self.gametype.as_ref()
    }

    pub fn empty(&self) -> bool {
        self.empty
    }

    pub fn full(&self) -> bool {
        self.full
    }
}

/// `getservers` message
#[derive(Debug, PartialEq, Eq)]
pub struct GetServersMessage {
    game_name: Option<GameName>,
    protocol_number: ProtocolNumber,
    filter_options: FilterOptions,
}

impl GetServersMessage {
    pub fn new(
        game_name: Option<GameName>,
        protocol_number: ProtocolNumber,
        filter_options: FilterOptions,
    ) -> Self {
        Self {
            game_name,
            protocol_number,
            filter_options,
        }
    }

    pub fn game_name(&self) -> Option<&GameName> {
        self.game_name.as_ref()
    }

    pub fn protocol_number(&self) -> ProtocolNumber {
        self.protocol_number
    }

    pub fn filter_options(&self) -> &FilterOptions {
        &self.filter_options
    }
}

/// `getserversResponse` message
#[derive(Debug, PartialEq, Eq)]
pub struct GetServersResponseMessage {
    servers: Vec<std::net::SocketAddrV4>,
    eot: bool,
}

impl GetServersResponseMessage {
    pub fn new(servers: Vec<std::net::SocketAddrV4>, eot: bool) -> Self {
        Self { servers, eot }
    }

    pub fn servers(&self) -> &[std::net::SocketAddrV4] {
        &self.servers[..]
    }

    pub fn eot(&self) -> bool {
        self.eot
    }
}

/// filter options for `getserversExt`
pub struct FilterExtOptions {
    /// `gametype=X` filter option
    gametype: Option<GameType>,
    /// empty servers option
    empty: bool,
    /// full servers option
    full: bool,
    // IPv4 servers option
    ipv4: bool,
    // IPv6 servers option
    ipv6: bool,
}

impl FilterExtOptions {
    pub fn new(
        gametype: Option<GameType>,
        empty: bool,
        full: bool,
        ipv4: bool,
        ipv6: bool,
    ) -> Self {
        Self {
            gametype,
            empty,
            full,
            ipv4,
            ipv6,
        }
    }

    pub fn gametype(&self) -> Option<&GameType> {
        self.gametype.as_ref()
    }

    pub fn empty(&self) -> bool {
        self.empty
    }

    pub fn full(&self) -> bool {
        self.full
    }

    pub fn ipv4(&self) -> bool {
        self.ipv4
    }

    pub fn ipv6(&self) -> bool {
        self.ipv6
    }
}

/// `getserversExt` message
pub struct GetServersExtMessage {
    game_name: GameName,
    protocol_number: ProtocolNumber,
    filter_options: FilterExtOptions,
}

impl GetServersExtMessage {
    pub fn new(
        game_name: GameName,
        protocol_number: ProtocolNumber,
        filter_options: FilterExtOptions,
    ) -> Self {
        Self {
            game_name,
            protocol_number,
            filter_options,
        }
    }

    pub fn game_name(&self) -> &GameName {
        &self.game_name
    }

    pub fn protocol_number(&self) -> ProtocolNumber {
        self.protocol_number
    }

    pub fn filter_options(&self) -> &FilterExtOptions {
        &self.filter_options
    }
}

/// `getserversExtResponse` message
pub struct GetServersExtResponseMessage {
    servers: Vec<std::net::SocketAddr>,
    eot: bool,
}

impl GetServersExtResponseMessage {
    pub fn new(servers: Vec<std::net::SocketAddr>, eot: bool) -> Self {
        Self { servers, eot }
    }

    pub fn servers(&self) -> &[std::net::SocketAddr] {
        &self.servers
    }

    pub fn eot(&self) -> bool {
        self.eot
    }
}
