pub mod deserializer;
pub mod error;
pub mod messages;
pub mod serializer;

pub use messages::{
    GameName, Gametype, GetServersExtResponseMessage, GetServersResponseMessage, HeartbeatMessage,
    ProtocolName,
};

pub use crate::error::ProtocolError;
/// [std::result::Result] alias with [ProtocolError] as `Err`
pub type Result<T> = std::result::Result<T, ProtocolError>;

#[cfg(test)]
mod tests {
    use super::deserializer::{getservers_message, heartbeat_message};
    use super::messages::{
        FilterOptions, GameName, GetServersMessage, HeartbeatMessage, ProtocolName,
    };
    use super::serializer::{gen_getservers_message, gen_heartbeat_message};
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
            FilterOptions::new(Some(b"0".to_vec()), true, true)
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
