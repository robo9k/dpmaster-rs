#![warn(missing_docs)]

//! Protocol datagram "messages" and related types
//!
//! The dpmaster protocol consists of a few messages that are passed between game servers and the master server to register a game server:
//! 1. [`heartbeat`](HeartbeatMessage)
//! 2. [`getinfo`](GetInfoMessage)
//! 3. [`infoResponse`](InfoResponseMessage)
//!
//! Then there are message that are passed between game clients and the master server to query game servers:
//! 1. [`getservers`](GetServersMessage)
//! 2. [`getserversResponse`](GetServersResponseMessage)
//!
//! To support [IPv6](https://en.wikipedia.org/wiki/IPv6) there are extended versions of the previous messages:
//! 1. [`getserversExt`](GetServersExtMessage)
//! 2. [`getserversExtResponse`](GetServersExtResponseMessage)

use crate::error::{EmptyError, InvalidByteError, InvalidChallengeError};
use crate::{ProtocolError, Result};

use memchr::memchr2;

fn is_ascii_printable(chr: u8) -> bool {
    chr >= 33 && chr <= 126
}

/// "Password" to authenticate messages
///
/// Contained in a [`getinfo` message](GetInfoMessage) and [`infoResponse` message](InfoResponseMessage).
///
/// The dpmaster protocol uses [UDP](https://en.wikipedia.org/wiki/User_Datagram_Protocol) which is spoofable so,
/// to authenticate datagrams and prevent denial-of-service in the ([`heartbeat`](HeartbeatMessage) →) [`getinfo`](GetInfoMessage) → [`infoResponse`](InfoResponseMessage) chain,
/// a "password" is used that should only be known to the game server and the master server.
#[derive(Debug, PartialEq, Eq)]
pub struct Challenge(Vec<u8>);

impl Challenge {
    /// Creates a new `Challenge` from a container of bytes.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dpmaster_proto::messages::Challenge;
    /// let challenge = Challenge::new(*b"A_ch4Lleng3")?;
    /// # Ok::<(), dpmaster_proto::error::InvalidChallengeError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return an [EmptyError](crate::error::EmptyError) if the supplied bytes are empty.
    /// ```rust
    /// # use dpmaster_proto::{error::InvalidChallengeError, messages::Challenge};
    /// #
    /// assert!(matches!(Challenge::new(*b"").unwrap_err(), InvalidChallengeError::Empty(..)));
    /// ```
    ///
    /// Will return [InvalidByteError](crate::error::InvalidByteError)
    /// if a supplied byte is not [ASCII](https://en.wikipedia.org/wiki/ASCII) printable (code 33 to 126)
    /// or is one of the disallowed characters `\`, `/`, `;`, `"` or `%`.
    /// ```rust
    /// # use dpmaster_proto::{error::InvalidChallengeError, messages::Challenge};
    /// #
    /// assert!(matches!(Challenge::new(*b"\xFF").unwrap_err(), InvalidChallengeError::InvalidByte(..)));
    /// assert!(matches!(Challenge::new(*b"uhoh;").unwrap_err(), InvalidChallengeError::InvalidByte(..)));
    /// ```
    pub fn new<T: Into<Vec<u8>>>(t: T) -> Result<Self, InvalidChallengeError> {
        let bytes = t.into();

        if bytes.is_empty() {
            return Err(EmptyError(()))?;
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
///
/// Sent from the master server to a game server in response to a [`heartbeat`](HeartbeatMessage) message from a game server.\
/// Responded to with a [`infoResponse` message](InfoResponseMessage) from the game server.
///
/// Contains a [`Challenge`](Challenge).
#[derive(Debug, PartialEq, Eq)]
pub struct GetInfoMessage {
    challenge: Challenge,
}

impl GetInfoMessage {
    /// Creates a new `GetInfoMessage` for the given `challenge`.
    pub fn new(challenge: Challenge) -> Self {
        Self { challenge }
    }

    /// Returns the `Challenge` contained in this message.
    pub fn challenge(&self) -> &Challenge {
        &self.challenge
    }
}

/// Maximum number of clients on a game server
///
/// Contained in the [`Info`](Info) of an [`infoResponse` message](InfoResponseMessage).
pub type MaxClientsNumber = std::num::NonZeroU32;

/// Current number of clients on a game server
///
/// Contained in the [`Info`](Info) of an [`infoResponse` message](InfoResponseMessage).
pub type ClientsNumber = u32;

/// Key in a [`Info`](Info) key-value pair
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct InfoKey(Vec<u8>);

impl InfoKey {
    /// Creates a new `InfoKey` from a container of bytes.
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

/// Value in a [`Info`](Info) key-value pair
#[derive(Debug, PartialEq, Eq)]
pub struct InfoValue(Vec<u8>);

impl InfoValue {
    /// Creates a new `InfoValue` from a container of bytes.
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

/// Map of [`InfoKey`](InfoKey)-[`InfoValue`](InfoValue) pairs
///
/// Contained in an [`infoResponse` message](InfoResponseMessage).
// TODO required and optional keys
#[derive(Debug, PartialEq, Eq)]
pub struct Info(indexmap::IndexMap<InfoKey, InfoValue>);

impl Info {
    // FIXME
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
///
/// Sent concludingly from a game server to the master server in response to a [`getinfo` message](GetInfoMessage) from the master server.
///
/// Contains [`Info`](Info) metadata.
#[derive(Debug, PartialEq, Eq)]
pub struct InfoResponseMessage {
    info: Info,
}

impl InfoResponseMessage {
    /// Creates a new `InfoResponseMessage` for the given `info`.
    pub fn new(info: Info) -> Self {
        Self { info }
    }

    /// Returns the `Info` contained in this message.
    pub fn info(&self) -> &Info {
        &self.info
    }
}

/// Protocol name
///
/// Contained in a [`heartbeat` message](HeartbeatMessage).
// TODO vs ProtocolNumber, GameName
#[derive(Debug, PartialEq, Eq)]
pub struct ProtocolName(Vec<u8>);

impl ProtocolName {
    /// Creates a new `ProtocolName` from a container of bytes.
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
///
/// Sent initially from game servers to the master server.\
/// Responded to with a [`getinfo` message](GetInfoMessage) from the master server.
///
/// Contains a [`ProtocolName`](ProtocolName).
#[derive(Debug, PartialEq, Eq)]
pub struct HeartbeatMessage {
    protocol_name: ProtocolName,
}

impl HeartbeatMessage {
    /// Creates a new `HeartbeatMessage` for the given `protocol_name`.
    pub fn new(protocol_name: ProtocolName) -> Self {
        Self { protocol_name }
    }

    /// Returns the `ProtocolName` contained in this message.
    pub fn protocol_name(&self) -> &ProtocolName {
        &self.protocol_name
    }
}

/// Protocol number
///
/// Contained in a [`getservers` message](GetServersMessage), [`getserversExt`](GetServersExtMessage)\
/// and in the [`Info`](Info) of an [`infoResponse` message](InfoResponseMessage).
// TODO vs ProtocolName, GameType
pub type ProtocolNumber = u32;

/// Game name
///
/// Contained in a [`getservers` message](GetServersMessage), [`getserversExt`](GetServersExtMessage)\
/// and in the [`Info`](Info) of an [`infoResponse` message](InfoResponseMessage).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GameName(Vec<u8>);

impl GameName {
    /// Creates a new `GameName` from a container of bytes.
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

/// Game type
///
/// Contained in the [`FilterOptions`](FilterOptions) of a [`getservers` message](GetServersMessage),
/// [`FilterExtOptions`](FilterExtOptions) of an [`getserversExt` message](GetServersExtMessage)\
/// and [`Info`](Info) of an [`infoResponse` message](InfoResponseMessage).
// TODO vs GameName, ProtocolNumber
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct GameType(Vec<u8>);

impl GameType {
    /// Creates a new `GameType` from a container of bytes.
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

/// Filter options for a [`getservers` message](GetServersMessage)
///
/// Contains a [`GameType`](GameType) and "empty" / "full" options.
///
/// IPv4-only variant of [`FilterExtOptions`](FilterExtOptions).
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
    /// Creates a new `FilterOptions` for the given `gametype`, `empty` / `full` options.
    pub fn new(gametype: Option<GameType>, empty: bool, full: bool) -> Self {
        Self {
            gametype,
            empty,
            full,
        }
    }

    /// Returns the `GameType` option contained in this filter.
    pub fn gametype(&self) -> Option<&GameType> {
        self.gametype.as_ref()
    }

    /// Returns the "empty" option contained in this filter.
    pub fn empty(&self) -> bool {
        self.empty
    }

    /// Returns the "full" option contained in this filter.
    pub fn full(&self) -> bool {
        self.full
    }
}

/// `getservers` message
///
/// Sent initially from a game client to the master server.\
/// Responded to with a [`getserversResponse` message](GetServersResponseMessage) from the master server.
///
/// Contains a [`GameName`](GameName), [`ProtocolNumber`](ProtocolNumber) and [`FilterOptions`](FilterOptions).
///
/// IPv4-only variant of the [`getserversExt` message](GetServersExtMessage).
#[derive(Debug, PartialEq, Eq)]
pub struct GetServersMessage {
    game_name: Option<GameName>,
    protocol_number: ProtocolNumber,
    filter_options: FilterOptions,
}

impl GetServersMessage {
    /// Creates a new `GetServersMessage` for the given `game_name`, `protocol_number` and `filter_options`.
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

    /// Returns the `GameName` contained in this message.
    pub fn game_name(&self) -> Option<&GameName> {
        self.game_name.as_ref()
    }

    /// Returns the `ProtocolNumber` contained in this message.
    pub fn protocol_number(&self) -> ProtocolNumber {
        self.protocol_number
    }

    /// Returns the `FilterOptions` contained in this message.
    pub fn filter_options(&self) -> &FilterOptions {
        &self.filter_options
    }
}

/// `getserversResponse` message
///
/// Sent concludingly from the master server to a game client in response to a [`getservers` message](GetServersMessage) from the game client.
///
/// Contains a list of [`SocketAddrV4`](std::net::SocketAddrV4) and End-of-Transmission flag.
///
/// IPv4-only variant of the [`getserversExtResponse` message](GetServersExtResponseMessage).
#[derive(Debug, PartialEq, Eq)]
pub struct GetServersResponseMessage {
    servers: Vec<std::net::SocketAddrV4>,
    eot: bool,
}

impl GetServersResponseMessage {
    /// Creates a new `GetServersResponseMessage` for the given `servers` and "eot" flag.
    pub fn new(servers: Vec<std::net::SocketAddrV4>, eot: bool) -> Self {
        Self { servers, eot }
    }

    /// Returns the server socket addresses contained in this message.
    pub fn servers(&self) -> &[std::net::SocketAddrV4] {
        &self.servers[..]
    }

    /// Returns the EOT flag contained in this message.
    pub fn eot(&self) -> bool {
        self.eot
    }
}

/// Filter options for a [`getserversExt` message](GetServersExtMessage)
///
/// Contains a [`GameType`](GameType), "empty" / "full" and "ipv4" / "ipv6" options.
///
/// IPv6-enabled variant of [`FilterOptions`](FilterOptions).
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
    /// Creates a new `FilterExtOptions` for the given `gametype`, `empty` / `full` and `ìpv4` / `ipv6` options.
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

    /// Returns the `GameType` option contained in this filter.
    pub fn gametype(&self) -> Option<&GameType> {
        self.gametype.as_ref()
    }

    /// Returns the "empty" option contained in this filter.
    pub fn empty(&self) -> bool {
        self.empty
    }

    /// Returns the "empty" option contained in this filter.
    pub fn full(&self) -> bool {
        self.full
    }

    /// Returns the "ipv4" option contained in this filter.
    pub fn ipv4(&self) -> bool {
        self.ipv4
    }

    /// Returns the "ipv6" option contained in this filter.
    pub fn ipv6(&self) -> bool {
        self.ipv6
    }
}

/// `getserversExt` message
///
/// Sent initially from a game client to the master server.\
/// Responded to with a [`getserversExtResponse` messsage](GetServersExtResponseMessage) from the master server.
///
/// Contains a [`GameName`](GameName), [`ProtocolNumber`](ProtocolNumber) and [`FilterExtOptions`](FilterExtOptions).
///
/// IPv6-enabled variant of the [`getservers` message](GetServersMessage).
pub struct GetServersExtMessage {
    game_name: GameName,
    protocol_number: ProtocolNumber,
    filter_options: FilterExtOptions,
}

impl GetServersExtMessage {
    /// Creates a new `GetServersMessage` for the given `game_name`, `protocol_number` and `filter_options`.
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

    /// Returns the `GameName` contained in this message.
    pub fn game_name(&self) -> &GameName {
        &self.game_name
    }

    /// Returns the `ProtocolNumber` contained in this message.
    pub fn protocol_number(&self) -> ProtocolNumber {
        self.protocol_number
    }

    /// Returns the `FilterExtOptions` contained in this message.
    pub fn filter_options(&self) -> &FilterExtOptions {
        &self.filter_options
    }
}

/// `getserversExtResponse` message
///
/// Sent concludingly from the master server to a game client in response to a [`getserversExt` message](GetServersExtMessage) from the game client.
///
/// Contains a list of [`SocketAddr`](std::net::SocketAddr) and End-of-Transmission flag.
///
/// IPv6-enabled variant of the [`getserversResponse` message](GetServersResponseMessage).
pub struct GetServersExtResponseMessage {
    servers: Vec<std::net::SocketAddr>,
    eot: bool,
}

impl GetServersExtResponseMessage {
    /// Creates a new `GetServersResponseMessage` for the given `servers` and "eot" flag.
    pub fn new(servers: Vec<std::net::SocketAddr>, eot: bool) -> Self {
        Self { servers, eot }
    }

    /// Returns the server socket addresses contained in this message.
    pub fn servers(&self) -> &[std::net::SocketAddr] {
        &self.servers
    }

    /// Returns the EOT flag contained in this message.
    pub fn eot(&self) -> bool {
        self.eot
    }
}
