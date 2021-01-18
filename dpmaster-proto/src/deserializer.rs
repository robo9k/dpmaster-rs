//! deserializer for messages

use crate::error::DeserializationError;
use crate::messages::{
    FilterOptions, GameName, Gametype, GetServersMessage, GetServersResponseMessage,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::is_digit;
use nom::combinator::opt;
use nom::multi::{many_till, separated_list0};
use nom::number::complete::{be_u16, be_u8};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use std::net::{Ipv4Addr, SocketAddrV4};

fn message_prefix(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"\xFF\xFF\xFF\xFF")(input)
}

fn getservers_command(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"getservers")(input)
}

fn is_space(chr: u8) -> bool {
    b' ' == chr
}

fn game_name(input: &[u8]) -> IResult<&[u8], Option<GameName>, DeserializationError<&[u8]>> {
    let (input, game_name) = opt(take_while1(|chr| !(is_digit(chr) || is_space(chr))))(input)?;
    Ok((input, game_name.map(|game_name| game_name.to_vec())))
}

fn protocol_number(input: &[u8]) -> IResult<&[u8], u32, DeserializationError<&[u8]>> {
    let (input, protocol_bytes) = take_while(is_digit)(input)?;
    let protocol_str = std::str::from_utf8(protocol_bytes).unwrap(); // TODO
    let protocol_number = u32::from_str_radix(protocol_str, 10).unwrap(); // TODO
    Ok((input, protocol_number))
}

enum FilterOption {
    Gametype(Gametype),
    Empty,
    Full,
}

fn filteroption_gametype(
    input: &[u8],
) -> IResult<&[u8], FilterOption, DeserializationError<&[u8]>> {
    let (input, gametype) = preceded(tag(b"gametype="), take_while1(|chr| chr != b' '))(input)?;
    Ok((input, FilterOption::Gametype(gametype.to_vec())))
}

fn filteroption_empty(input: &[u8]) -> IResult<&[u8], FilterOption, DeserializationError<&[u8]>> {
    let (input, _) = tag(b"empty")(input)?;
    Ok((input, FilterOption::Empty))
}

fn filteroption_full(input: &[u8]) -> IResult<&[u8], FilterOption, DeserializationError<&[u8]>> {
    let (input, _) = tag(b"full")(input)?;
    Ok((input, FilterOption::Full))
}

fn filteroption(input: &[u8]) -> IResult<&[u8], FilterOption, DeserializationError<&[u8]>> {
    alt((filteroption_gametype, filteroption_empty, filteroption_full))(input)
}

fn filteroptions(input: &[u8]) -> IResult<&[u8], FilterOptions, DeserializationError<&[u8]>> {
    let mut gametype: Option<Gametype> = None;
    let mut empty: bool = false;
    let mut full: bool = false;

    let (input, filteroptions) = separated_list0(tag(b" "), filteroption)(input)?;
    for filteroption in filteroptions {
        match filteroption {
            FilterOption::Gametype(g) => {
                gametype = Some(g);
            }
            FilterOption::Empty => {
                empty = true;
            }
            FilterOption::Full => {
                full = true;
            }
        }
    }

    Ok((input, FilterOptions::new(gametype, empty, full)))
}

fn getservers_payload(
    input: &[u8],
) -> IResult<&[u8], GetServersMessage, DeserializationError<&[u8]>> {
    let (input, (_, game_name, _, protocol_number, _, filteroptions)) = tuple((
        take_while1(is_space),
        game_name,
        take_while(is_space),
        protocol_number,
        take_while(is_space),
        filteroptions,
    ))(input)?;
    Ok((
        input,
        GetServersMessage::new(game_name, protocol_number, filteroptions),
    ))
}

pub fn getservers(input: &[u8]) -> IResult<&[u8], GetServersMessage, DeserializationError<&[u8]>> {
    preceded(getservers_command, getservers_payload)(input)
}

pub fn getservers_message(
    input: &[u8],
) -> IResult<&[u8], GetServersMessage, DeserializationError<&[u8]>> {
    preceded(message_prefix, getservers)(input)
}

fn socketaddr4(input: &[u8]) -> IResult<&[u8], SocketAddrV4, DeserializationError<&[u8]>> {
    let (input, (a, b, c, d, port)) = tuple((be_u8, be_u8, be_u8, be_u8, be_u16))(input)?;
    let ipv4addr = Ipv4Addr::new(a, b, c, d);
    let socketaddrv4 = SocketAddrV4::new(ipv4addr, port);
    Ok((input, socketaddrv4))
}

fn socketaddr4_separator(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"\\")(input)
}

fn eot(input: &[u8]) -> IResult<&[u8], bool, DeserializationError<&[u8]>> {
    match input {
        b"\\EOT\0\0\0" => Ok((&input[7..], true)),
        b"" => Ok((input, false)),
        _ => Err(nom::Err::Error(nom::error::make_error(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn getserversresponse_payload(
    input: &[u8],
) -> IResult<&[u8], GetServersResponseMessage, DeserializationError<&[u8]>> {
    let (input, (servers, eot)) =
        many_till(preceded(socketaddr4_separator, socketaddr4), eot)(input)?;
    let getserversresponse = GetServersResponseMessage::new(servers, eot);
    Ok((input, getserversresponse))
}

fn getserversresponse_command(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"getserversResponse")(input)
}

pub fn getserversresponse(
    input: &[u8],
) -> IResult<&[u8], GetServersResponseMessage, DeserializationError<&[u8]>> {
    preceded(getserversresponse_command, getserversresponse_payload)(input)
}

pub fn getserversresponse_message(
    input: &[u8],
) -> IResult<&[u8], GetServersResponseMessage, DeserializationError<&[u8]>> {
    preceded(message_prefix, getserversresponse)(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_getservers_message_q3a() {
        let data = &b"getservers 67 gametype=0 empty full"[..];
        let result = getservers(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetServersMessage::new(
                    None,
                    67,
                    FilterOptions::new(Some(b"0".to_vec()), true, true)
                )
            ))
        );
    }

    #[test]
    fn test_getservers_message_woet() {
        let data = &b"getservers 84"[..];
        let result = getservers(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetServersMessage::new(None, 84, FilterOptions::new(None, false, false))
            ))
        );
    }

    #[test]
    fn test_getservers_message_nexuiz() {
        let data = &b"getservers Nexuiz 3"[..];
        let result = getservers(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetServersMessage::new(
                    Some(b"Nexuiz".to_vec()),
                    3,
                    FilterOptions::new(None, false, false)
                )
            ))
        );
    }

    #[test]
    fn test_getservers_message_qfusion() {
        let data = &b"getservers qfusion 39 full"[..];
        let result = getservers(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetServersMessage::new(
                    Some(b"qfusion".to_vec()),
                    39,
                    FilterOptions::new(None, false, true)
                )
            ))
        );
    }

    #[test]
    fn test_getserversresponse_multiple() {
        let data = &b"getserversResponse\\\xC0\x00\x02\x01\x6D\x38\\\xC6\x33\x64\x02\x6D\x39\\\xCB\x00\x71\x03\x6D\x3A"[..];
        let result = getserversresponse(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetServersResponseMessage::new(
                    vec![
                        SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 27960),
                        SocketAddrV4::new(Ipv4Addr::new(198, 51, 100, 2), 27961),
                        SocketAddrV4::new(Ipv4Addr::new(203, 0, 113, 3), 27962),
                    ],
                    false
                )
            ))
        );
    }

    #[test]
    fn test_getserversresponse_eot() {
        let data = &b"getserversResponse\\\x01\x02\x03\x04\x08\x00\\EOT\0\0\0"[..];
        let result = getserversresponse(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetServersResponseMessage::new(
                    vec![SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 2048),],
                    true
                )
            ))
        );
    }
}
