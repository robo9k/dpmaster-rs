//#![warn(missing_docs)]

//! `dpmaster` protocol
//!
//! ## `dpmaster`
//!
//! `dpmaster`, "an open master server",
//! is both a generic protocol to register game servers with a master server and query them from game clients
//! as well as a reference master server implementation written in the C programming language.
//!
//! This crate implements the `dpmaster` wire protocol as defined in its ["Technical information" documentation](https://hg.icculus.org/molivier/dpmaster/file/tip/doc/techinfo.txt).
//!
//! ## `dpmaster` protocol
//!
//! The `dpmaster` protocol is a custom UDP protocol where datagram packets contain a mix of binary delimiters as well as ASCII text.\
//! Neither the protocol nor its specification are versioned, but they have not changed for a couple of years.
//!
//! This crate implements (de)serialization of (not-yet all) packets, which the spec calls messages:
//! * [`heartbeat` message](crate::messages::HeartbeatMessage)
//! * [`getinfo` message](crate::messages::GetInfoMessage)
//! * [`infoResponse` message](crate::messages::InfoResponseMessage)
//! * [`getservers` message](crate::messages::GetServersMessage)
//! * [`getserversResponse` message](crate::messages::GetServersResponseMessage)
//! * [`getserversExt` message](crate::messages::GetServersExtMessage)
//! * [`getserversExtResponse` message](crate::messages::GetServersExtResponseMessage)
//!
//!
//! ## Message flows
//!
//! The "master server" concept is that there is a central server instance, e.g. at `master.example:27950`, that all game servers report to.\
//! Game servers then register themselves with the master server, e.g. their IPs `192.0.2.1:27960` / `[2001:db8::1]:27964` aswell as some [`Info`](crate::messages::Info) metadata.\
//! Game clients query the master server for a (filtered) list of game servers.
//!
//! ### `heartbeat` message flow
//!
//! The "heartbeat" message flow registers a game server with the master server.\
//! It should be initiated by game servers when their configuration changes and periodically to remain registered with the master server.
//!
//! ❶ Game server sends `heartbeat` message to master server
//!
//! ❷ Master server sends `getinfo` message with new challenge back to game server
//!
//! ❸ Game server sends `infoResponse` message with same challenge back to master server
#![doc=include_str!("../assets/message-flow-heartbeat.svg")]
//!
//! ### `getservers` message flow
//!
//! The "getservers" message flow queries a master server for game servers.\
//! It is initiated by game clients e.g. when they want to display a list of servers in their UI so players can decide which one to connect to.
//!
//! ❶ Game client sends `getservers` message to master server
//!
//! ❷ Master server sends `getserversResponse` message(s) back to game client
#![doc=include_str!("../assets/message-flow-getservers.svg")]
//!
//! ## `dpmaster-proto` crate
//!
//! This crate implements the `dpmaster` protocol wire format, i.e. it allows to serialize and deserialize protocol messages as bytes to and from UDP packets.\
//! It does however not contain logic (["sans I/O"](https://sans-io.readthedocs.io/)), e.g. to implement a master server, game server or game client.\
//!
//! The `dpmaster-codec` crate implements Tokio codecs on top of this protocol crate.\
//! The `dpmaster-game-client-bin` crate implements a "game client" on top of a codec in form of a command-line-interface to query a master server for game servers.

pub mod deserializer;
pub mod error;
pub mod messages;
pub mod serializer;

pub use messages::{
    Challenge, GameName, GameType, GetInfoMessage, GetServersExtResponseMessage,
    GetServersResponseMessage, HeartbeatMessage, Info, InfoKey, InfoResponseMessage, InfoValue,
    ProtocolName,
};

pub use crate::error::ProtocolError;
/// [std::result::Result] alias with [ProtocolError] as `Err`
pub type Result<T, E = ProtocolError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::deserializer::{
        getinfo_message, getservers_message, heartbeat_message, inforesponse_message,
    };
    use super::messages::{
        Challenge, FilterOptions, GameName, GameType, GetInfoMessage, GetServersMessage,
        HeartbeatMessage, Info, InfoKey, InfoResponseMessage, InfoValue, ProtocolName,
    };
    use super::serializer::{
        gen_getinfo_message, gen_getservers_message, gen_heartbeat_message,
        gen_inforesponse_message,
    };
    use cookie_factory::gen_simple;
    use std::io::Cursor;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    macro_rules! roundtrip_message_test {
        (
            $name:ident {
                message: $message:expr,
                serializer: $serializer:expr,
                deserializer: $deserializer:expr,
            }
        ) => {
            #[test]
            fn $name() {
                let message = $message;
                let serialized_message = $serializer(&message);

                let mut buffer = [0u8; 512];
                let cursor = Cursor::new(&mut buffer[..]);
                let cursor = gen_simple(serialized_message, cursor).unwrap();
                let size = cursor.position() as usize;
                let buffer = cursor.into_inner();

                let (_, deserialized_message) = $deserializer(&buffer[..size]).unwrap();

                assert_eq!(deserialized_message, message);
            }
        };
    }

    macro_rules! roundtrip_heartbeat_message_test {
        (
        $name:ident {
            message: $message:expr
        }
        ) => {
            roundtrip_message_test!($name {
                message: $message,
                serializer: gen_heartbeat_message,
                deserializer: heartbeat_message,
            });
        };
    }

    roundtrip_heartbeat_message_test!(test_roundtrip_heartbeat_message_dp {
        message: HeartbeatMessage::new(ProtocolName::new(b"DarkPlaces".to_vec()).unwrap(),)
    });

    roundtrip_heartbeat_message_test!(test_roundtrip_heartbeat_message_q3a {
        message: HeartbeatMessage::new(ProtocolName::new(b"QuakeArena-1".to_vec()).unwrap(),)
    });

    roundtrip_heartbeat_message_test!(test_roundtrip_heartbeat_message_rtcw {
        message: HeartbeatMessage::new(ProtocolName::new(b"Wolfenstein-1".to_vec()).unwrap(),)
    });

    roundtrip_heartbeat_message_test!(test_roundtrip_heartbeat_message_woet {
        message: HeartbeatMessage::new(ProtocolName::new(b"EnemyTerritory-1".to_vec()).unwrap(),)
    });

    macro_rules! roundtrip_getinfo_message_test {
        (
        $name:ident {
            message: $message:expr
        }
        ) => {
            roundtrip_message_test!($name {
                message: $message,
                serializer: gen_getinfo_message,
                deserializer: getinfo_message,
            });
        };
    }

    roundtrip_getinfo_message_test!(test_roundtrip_getinfo_message {
        message: GetInfoMessage::new(Challenge::new(b"A_ch4Lleng3".to_vec()).unwrap(),)
    });

    macro_rules! roundtrip_inforesponse_message_test {
        (
        $name:ident {
            message: $message:expr
        }
        ) => {
            roundtrip_message_test!($name {
                message: $message,
                serializer: gen_inforesponse_message,
                deserializer: inforesponse_message,
            });
        };
    }

    roundtrip_inforesponse_message_test!(test_roundtrip_inforesponse_message {
        message: InfoResponseMessage::new({
            let mut info = Info::new();
            info.insert(
                InfoKey::new(b"sv_maxclients".to_vec()).unwrap(),
                InfoValue::new(b"8".to_vec()).unwrap(),
            );
            info.insert(
                InfoKey::new(b"clients".to_vec()).unwrap(),
                InfoValue::new(b"0".to_vec()).unwrap(),
            );
            info
        })
    });

    macro_rules! roundtrip_getservers_message_test {
        (
        $name:ident {
            message: $message:expr
        }
        ) => {
            roundtrip_message_test!($name {
                message: $message,
                serializer: gen_getservers_message,
                deserializer: getservers_message,
            });
        };
    }

    roundtrip_getservers_message_test!(test_roundtrip_getservers_message_q3a {
        message: GetServersMessage::new(
            None,
            67,
            FilterOptions::new(Some(GameType::new(b"0".to_vec()).unwrap()), true, true)
        )
    });

    roundtrip_getservers_message_test!(test_roundtrip_getservers_message_woet {
        message: GetServersMessage::new(None, 84, FilterOptions::new(None, false, false))
    });

    roundtrip_getservers_message_test!(test_roundtrip_getservers_message_nexuiz {
        message: GetServersMessage::new(
            Some(GameName::new(b"Nexuiz".to_vec()).unwrap()),
            3,
            FilterOptions::new(None, false, false)
        )
    });

    roundtrip_getservers_message_test!(test_roundtrip_getservers_message_qfusion {
        message: GetServersMessage::new(
            Some(GameName::new(b"qfusion".to_vec()).unwrap()),
            39,
            FilterOptions::new(None, false, true)
        )
    });
}
