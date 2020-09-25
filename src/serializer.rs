//! serializer for messages

use crate::messages::{FilterOption, GameName, GetServersMessage, ProtocolNumber};
use cookie_factory::combinator::{cond, slice, string};
use cookie_factory::multi::separated_list;
use cookie_factory::sequence::tuple;
use cookie_factory::{SerializeFn, WriteContext};
use std::io::Write;

fn gen_message_prefix<W: Write>() -> impl SerializeFn<W> {
    slice(b"\xFF\xFF\xFF\xFF")
}

fn gen_game_name<'a, 'b: 'a, W: Write + 'a>(game_name: &'b GameName) -> impl SerializeFn<W> + 'a {
    slice(&game_name[..])
}

fn gen_protocol_number<W: Write>(protocol_number: ProtocolNumber) -> impl SerializeFn<W> {
    string(protocol_number.to_string())
}

fn gen_filter_option<'a, W: Write>(filter_option: &'a FilterOption) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match filter_option {
        FilterOption::Gametype(ref x) => tuple((slice(b"gametype="), slice(x)))(out),
        FilterOption::Empty => slice(b"empty")(out),
        FilterOption::Full => slice(b"full")(out),
        FilterOption::IPv4 => slice(b"ipv4")(out),
        FilterOption::IPv6 => slice(b"ipv6")(out),
    }
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
        cond(
            !message.filter_options().is_empty(),
            tuple((
                slice(b" "),
                separated_list(
                    slice(b" "),
                    message.filter_options().iter().map(gen_filter_option),
                ),
            )),
        ),
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
            vec![
                FilterOption::Gametype(b"0".to_vec()),
                FilterOption::Empty,
                FilterOption::Full,
            ],
        ),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers 67 gametype=0 empty full"[..]
    });

    gen_message_test!(test_gen_getservers_message_woet {
        message: GetServersMessage::new(None, 84, vec![],),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers 84"[..]
    });

    gen_message_test!(test_gen_getservers_message_nexuiz {
        message: GetServersMessage::new(Some(b"Nexuiz".to_vec()), 3, vec![],),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers Nexuiz 3"[..]
    });

    gen_message_test!(test_gen_getservers_message_qfusion {
        message: GetServersMessage::new(Some(b"qfusion".to_vec()), 39, vec![FilterOption::Full],),
        function: gen_getservers_message,
        buffer: &b"\xFF\xFF\xFF\xFFgetservers qfusion 39 full"[..]
    });
}
