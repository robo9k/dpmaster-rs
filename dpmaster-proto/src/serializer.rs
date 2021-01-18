//! serializer for messages

use crate::messages::{
    FilterOptions, GameName, Gametype, GetServersMessage, GetServersResponseMessage, ProtocolNumber,
};
use cookie_factory::bytes::{be_u16, be_u8};
use cookie_factory::combinator::{cond, slice, string};
use cookie_factory::multi::many_ref;
use cookie_factory::sequence::tuple;
use cookie_factory::{SerializeFn, WriteContext};
use std::io::Write;

fn gen_message_prefix<W: Write>() -> impl SerializeFn<W> {
    slice(b"\xFF\xFF\xFF\xFF")
}

fn gen_game_name<'a, 'b: 'a, W: Write + 'a>(game_name: &'b GameName) -> impl SerializeFn<W> + 'a {
    slice(&game_name[..])
}

fn gen_gametype<'a, 'b: 'a, W: Write + 'a>(gametype: &'b Gametype) -> impl SerializeFn<W> + 'a {
    slice(&gametype[..])
}

fn gen_protocol_number<W: Write>(protocol_number: ProtocolNumber) -> impl SerializeFn<W> {
    string(protocol_number.to_string())
}

fn gen_filter_options<'a, 'b: 'a, W: Write + 'a>(
    filter_options: &'b FilterOptions,
) -> impl SerializeFn<W> + 'a {
    tuple((
        move |out: WriteContext<W>| match filter_options.gametype() {
            Some(gametype) => {
                tuple((slice(b" "), slice(b"gametype="), gen_gametype(gametype)))(out)
            }
            None => Ok(out),
        },
        cond(filter_options.empty(), slice(b" empty")),
        cond(filter_options.full(), slice(b" full")),
    ))
}

pub fn gen_getservers_message<'a, 'b: 'a, W: Write + 'a>(
    message: &'b GetServersMessage,
) -> impl SerializeFn<W> + 'a {
    tuple((
        gen_message_prefix(),
        slice(b"getservers "),
        move |out: WriteContext<W>| match message.game_name() {
            Some(game_name) => tuple((gen_game_name(game_name), slice(b" ")))(out),
            None => Ok(out),
        },
        gen_protocol_number(message.protocol_number()),
        gen_filter_options(message.filter_options()),
    ))
}

fn gen_socketaddrv4<'a, 'b: 'a, W: Write + 'a>(
    addr: &'b std::net::SocketAddrV4,
) -> impl SerializeFn<W> + 'a {
    let octets = addr.ip().octets();
    move |out: WriteContext<W>| {
        tuple((
            slice(b"\\"),
            many_ref(&octets[..], |&i| be_u8(i)),
            be_u16(addr.port()),
        ))(out)
    }
}

pub fn gen_getserversresponse_message<'a, 'b: 'a, W: Write + 'a>(
    message: &'b GetServersResponseMessage,
) -> impl SerializeFn<W> + 'a {
    tuple((
        gen_message_prefix(),
        slice(b"getserversResponse"),
        many_ref(message.servers(), gen_socketaddrv4),
        cond(message.eot(), slice(b"\\EOT\0\0\0")),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cookie_factory::gen_simple;
    use std::io::Cursor;

    macro_rules! gen_message_test {
        (
            $name:ident {
                message: $message:expr,
                function: $function:expr,
                buffer: $buf:expr
            }
        ) => {
            #[test]
            fn $name() {
                let message = $message;
                let sr = $function(&message);

                let mut buffer = [0u8; 512];
                let cursor = Cursor::new(&mut buffer[..]);
                let cursor = gen_simple(sr, cursor).unwrap();
                let size = cursor.position() as usize;
                let buffer = cursor.into_inner();

                assert_eq!(&buffer[..size], $buf);
            }
        };
    }

    gen_message_test!(test_gen_getservers_message_q3a {
        message: GetServersMessage::new(
            None,
            67,
            FilterOptions::new(Some(b"0".to_vec()), true, true),
        ),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers 67 gametype=0 empty full"[..]
    });

    gen_message_test!(test_gen_getservers_message_woet {
        message: GetServersMessage::new(None, 84, FilterOptions::new(None, false, false),),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers 84"[..]
    });

    gen_message_test!(test_gen_getservers_message_nexuiz {
        message: GetServersMessage::new(
            Some(GameName::new(b"Nexuiz".to_vec()).unwrap()),
            3,
            FilterOptions::new(None, false, false),
        ),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers Nexuiz 3"[..]
    });

    gen_message_test!(test_gen_getservers_message_qfusion {
        message: GetServersMessage::new(
            Some(GameName::new(b"qfusion".to_vec()).unwrap()),
            39,
            FilterOptions::new(None, false, true)
        ),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers qfusion 39 full"[..]
    });

    gen_message_test!(test_gen_getserversresponse_message {
        message: GetServersResponseMessage::new(vec!["1.2.3.4:2048".parse().unwrap()], true),
        function: gen_getserversresponse_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetserversResponse\\\x01\x02\x03\x04\x08\x00\\EOT\0\0\0"[..]
    });
}
