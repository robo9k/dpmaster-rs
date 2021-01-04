//! protocol messages

/// protocol number
pub type ProtocolNumber = u32;

/// game name
pub type GameName = Vec<u8>;

/// game type
pub type Gametype = Vec<u8>;

/// filter options for `getservers`
#[derive(Debug, PartialEq, Eq)]
pub struct FilterOptions {
    /// `gametype=X` filter option
    gametype: Option<Gametype>,
    /// empty servers option
    empty: bool,
    /// full servers option
    full: bool,
}

impl FilterOptions {
    pub fn new(gametype: Option<Vec<u8>>, empty: bool, full: bool) -> Self {
        Self {
            gametype,
            empty,
            full,
        }
    }

    pub fn gametype(&self) -> Option<&Gametype> {
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
    gametype: Option<Gametype>,
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
        gametype: Option<Gametype>,
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

    pub fn gametype(&self) -> Option<&Gametype> {
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
        self.game_name.as_ref()
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
