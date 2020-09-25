//! protocol messages

/// protocol number
pub type ProtocolNumber = u32;

/// game name
pub type GameName = Vec<u8>;

/// filter option
pub enum FilterOption {
    /// `gametype=X` filter option
    Gametype(Vec<u8>),
    /// empty servers option
    Empty,
    /// full servers option
    Full,
    /// IPv4 servers option
    IPv4,
    /// IPv6 servers option
    IPv6,
}

/// `getservers` message
pub struct GetServersMessage {
    game_name: Option<GameName>,
    protocol_number: ProtocolNumber,
    filter_options: Vec<FilterOption>,
}

impl GetServersMessage {
    pub fn new(
        game_name: Option<GameName>,
        protocol_number: ProtocolNumber,
        filter_options: Vec<FilterOption>,
    ) -> Self {
        Self {
            game_name,
            protocol_number,
            filter_options,
        }
    }

    pub fn game_name(self) -> Option<GameName> {
        self.game_name
    }

    pub fn protocol_number(self) -> ProtocolNumber {
        self.protocol_number
    }

    pub fn filter_options(self) -> Vec<FilterOption> {
        self.filter_options
    }
}

/// `getserversResponse` message
pub struct GetServersResponse {
    servers: Vec<std::net::SocketAddrV4>,
    eot: bool,
}

impl GetServersResponse {
    pub fn new(servers: Vec<std::net::SocketAddrV4>, eot: bool) -> Self {
        Self { servers, eot }
    }

    pub fn servers(self) -> Vec<std::net::SocketAddrV4> {
        self.servers
    }

    pub fn eot(self) -> bool {
        self.eot
    }
}

/// `getserversExt` message
pub struct GetServersExtMessage {
    game_name: GameName,
    protocol_number: ProtocolNumber,
    filter_options: Vec<FilterOption>,
}

impl GetServersExtMessage {
    pub fn new(
        game_name: GameName,
        protocol_number: ProtocolNumber,
        filter_options: Vec<FilterOption>,
    ) -> Self {
        Self {
            game_name,
            protocol_number,
            filter_options,
        }
    }

    pub fn game_name(self) -> GameName {
        self.game_name
    }

    pub fn protocol_number(self) -> ProtocolNumber {
        self.protocol_number
    }

    pub fn filter_options(self) -> Vec<FilterOption> {
        self.filter_options
    }
}

/// `getserversExtResponse` message
pub struct GetServersExtResponse {
    servers: Vec<std::net::SocketAddr>,
    eot: bool,
}

impl GetServersExtResponse {
    pub fn new(servers: Vec<std::net::SocketAddr>, eot: bool) -> Self {
        Self { servers, eot }
    }

    pub fn servers(self) -> Vec<std::net::SocketAddr> {
        self.servers
    }

    pub fn eot(self) -> bool {
        self.eot
    }
}
