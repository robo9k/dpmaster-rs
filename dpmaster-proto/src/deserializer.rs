//! deserializer for messages

use crate::error::DeserializationError;
use crate::messages::{
    Challenge, FilterOptions, GameName, GameType, GetInfoMessage, GetServersMessage,
    GetServersResponseMessage, HeartbeatMessage, Info, InfoKey, InfoResponseMessage, InfoValue,
    ProtocolName,
};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::{is_digit, is_newline};
use nom::combinator::{opt, rest};
use nom::error::context;
use nom::multi::{many1, many_till, separated_list0};
use nom::number::complete::{be_u16, be_u8};
use nom::sequence::{preceded, tuple};
use nom::IResult;
use std::net::{Ipv4Addr, SocketAddrV4};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ErrorKind {
    MessagePrefix,
}

pub trait ParseError<I>: nom::error::ParseError<I> {
    fn from_dpmaster_error_kind(input: I, kind: ErrorKind) -> Self;
    fn append_dpmaster(input: I, kind: ErrorKind, other: Self) -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub enum VerboseErrorKind {
    Context(&'static str),
    Char(char),
    Nom(nom::error::ErrorKind),
    Dpmaster(ErrorKind),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VerboseError<I> {
    pub errors: Vec<(I, VerboseErrorKind)>,
}

impl<I> nom::error::ParseError<I> for VerboseError<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        VerboseError {
            errors: vec![(input, VerboseErrorKind::Nom(kind))],
        }
    }

    fn append(input: I, kind: nom::error::ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Nom(kind)));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        VerboseError {
            errors: vec![(input, VerboseErrorKind::Char(c))],
        }
    }
}

impl<I> ParseError<I> for () {
    fn from_dpmaster_error_kind(_: I, _: ErrorKind) -> Self {}

    fn append_dpmaster(_: I, _: ErrorKind, _: Self) -> Self {}
}

impl<I> ParseError<I> for VerboseError<I> {
    fn from_dpmaster_error_kind(input: I, kind: ErrorKind) -> Self {
        VerboseError {
            errors: vec![(input, VerboseErrorKind::Dpmaster(kind))],
        }
    }

    fn append_dpmaster(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Dpmaster(kind)));
        other
    }
}

impl<I> nom::error::ContextError<I> for VerboseError<I> {
    fn add_context(input: I, ctx: &'static str, mut other: Self) -> Self {
        other.errors.push((input, VerboseErrorKind::Context(ctx)));
        other
    }
}

impl<I: std::fmt::Display> std::fmt::Display for VerboseError<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Parse error:")?;
        for (input, error) in &self.errors {
            match error {
                VerboseErrorKind::Dpmaster(e) => writeln!(f, "{:?} at: {}", e, input)?,
                VerboseErrorKind::Nom(e) => writeln!(f, "{:?} at: {}", e, input)?,
                VerboseErrorKind::Char(c) => writeln!(f, "expected '{}' at: {}", c, input)?,
                VerboseErrorKind::Context(s) => writeln!(f, "in section '{}', at: {}", s, input)?,
            }
        }

        Ok(())
    }
}

fn append<I: Clone, E: ParseError<I>, F, O>(
    kind: ErrorKind,
    mut f: F,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    F: nom::Parser<I, O, E>,
{
    move |i: I| match f.parse(i.clone()) {
        Ok(o) => Ok(o),
        Err(nom::Err::Incomplete(i)) => Err(nom::Err::Incomplete(i)),
        Err(nom::Err::Error(e)) => Err(nom::Err::Error(E::append_dpmaster(i, kind, e))),
        Err(nom::Err::Failure(e)) => Err(nom::Err::Failure(E::append_dpmaster(i, kind, e))),
    }
}

/// Parser for the `\xFF\xFF\xFF\xFF` message prefix
pub fn message_prefix<'a, Error>(input: &'a [u8]) -> nom::IResult<&'a [u8], &'a [u8], Error>
where
    Error: ParseError<&'a [u8]> + nom::error::ContextError<&'a [u8]>,
{
    context(
        "message prefix",
        append(ErrorKind::MessagePrefix, tag(b"\xFF\xFF\xFF\xFF")),
    )(input)
}

fn protocol_name(input: &[u8]) -> IResult<&[u8], ProtocolName, DeserializationError<&[u8]>> {
    let (input, protocol_name) = take_while1(|chr| !(is_newline(chr)))(input)?;
    Ok((input, ProtocolName::new(protocol_name.to_vec()).unwrap())) // TODO
}

fn heartbeat_command(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"heartbeat")(input)
}

fn heartbeat_payload(
    input: &[u8],
) -> IResult<&[u8], HeartbeatMessage, DeserializationError<&[u8]>> {
    let (input, (_, protocol_name, _)) =
        tuple((take_while1(is_space), protocol_name, take_while(is_newline)))(input)?;
    Ok((input, HeartbeatMessage::new(protocol_name)))
}

pub fn heartbeat(input: &[u8]) -> IResult<&[u8], HeartbeatMessage, DeserializationError<&[u8]>> {
    preceded(heartbeat_command, heartbeat_payload)(input)
}

pub fn heartbeat_message(
    input: &[u8],
) -> IResult<&[u8], HeartbeatMessage, DeserializationError<&[u8]>> {
    preceded(message_prefix, heartbeat)(input)
}

fn getinfo_command(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"getinfo")(input)
}

fn challenge(input: &[u8]) -> IResult<&[u8], Challenge, DeserializationError<&[u8]>> {
    let (input, challenge) = rest(input)?;
    Ok((input, Challenge::new(challenge.to_vec()).unwrap())) // TODO
}

fn getinfo_payload(input: &[u8]) -> IResult<&[u8], GetInfoMessage, DeserializationError<&[u8]>> {
    let (input, (_, challenge)) = tuple((take_while1(is_space), challenge))(input)?;
    Ok((input, GetInfoMessage::new(challenge)))
}

pub fn getinfo(input: &[u8]) -> IResult<&[u8], GetInfoMessage, DeserializationError<&[u8]>> {
    preceded(getinfo_command, getinfo_payload)(input)
}

pub fn getinfo_message(
    input: &[u8],
) -> IResult<&[u8], GetInfoMessage, DeserializationError<&[u8]>> {
    preceded(message_prefix, getinfo)(input)
}

fn inforesponse_command(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"infoResponse")(input)
}

fn info_key(input: &[u8]) -> IResult<&[u8], InfoKey, DeserializationError<&[u8]>> {
    let (input, k) = take_while1(|chr| b'\\' != chr)(input)?;
    Ok((input, InfoKey::new(k.to_vec()).unwrap())) // TODO
}

fn info_value(input: &[u8]) -> IResult<&[u8], InfoValue, DeserializationError<&[u8]>> {
    let (input, v) = take_while1(|chr| b'\\' != chr)(input)?;
    Ok((input, InfoValue::new(v.to_vec()).unwrap())) // TODO
}

fn info_kv(input: &[u8]) -> IResult<&[u8], (InfoKey, InfoValue), DeserializationError<&[u8]>> {
    let (input, (_, k, _, v)) = tuple((tag(b"\\"), info_key, tag(b"\\"), info_value))(input)?;
    Ok((input, (k, v)))
}

fn info(input: &[u8]) -> IResult<&[u8], Info, DeserializationError<&[u8]>> {
    let (input, kv) = many1(info_kv)(input)?;
    let mut info = Info::new();
    for (key, value) in kv {
        info.insert(key, value);
    }
    Ok((input, info))
}

fn inforesponse_payload(
    input: &[u8],
) -> IResult<&[u8], InfoResponseMessage, DeserializationError<&[u8]>> {
    let (input, (_, info)) = tuple((tag(b"\n"), info))(input)?;
    Ok((input, InfoResponseMessage::new(info)))
}

pub fn inforesponse(
    input: &[u8],
) -> IResult<&[u8], InfoResponseMessage, DeserializationError<&[u8]>> {
    preceded(inforesponse_command, inforesponse_payload)(input)
}

pub fn inforesponse_message(
    input: &[u8],
) -> IResult<&[u8], InfoResponseMessage, DeserializationError<&[u8]>> {
    preceded(message_prefix, inforesponse)(input)
}

fn getservers_command(input: &[u8]) -> IResult<&[u8], &[u8], DeserializationError<&[u8]>> {
    tag(b"getservers")(input)
}

fn is_space(chr: u8) -> bool {
    b' ' == chr
}

fn game_name(input: &[u8]) -> IResult<&[u8], Option<GameName>, DeserializationError<&[u8]>> {
    let (input, game_name) = opt(take_while1(|chr| !(is_digit(chr) || is_space(chr))))(input)?;
    Ok((
        input,
        game_name.map(|game_name| GameName::new(game_name.to_vec()).unwrap()),
    )) // TODO
}

fn protocol_number(input: &[u8]) -> IResult<&[u8], u32, DeserializationError<&[u8]>> {
    let (input, protocol_bytes) = take_while(is_digit)(input)?;
    let protocol_str = std::str::from_utf8(protocol_bytes).unwrap(); // TODO
    let protocol_number = u32::from_str_radix(protocol_str, 10).unwrap(); // TODO
    Ok((input, protocol_number))
}

enum FilterOption {
    GameType(GameType),
    Empty,
    Full,
}

fn filteroption_gametype(
    input: &[u8],
) -> IResult<&[u8], FilterOption, DeserializationError<&[u8]>> {
    let (input, gametype) = preceded(tag(b"gametype="), take_while1(|chr| chr != b' '))(input)?;
    Ok((
        input,
        FilterOption::GameType(GameType::new(gametype.to_vec()).unwrap()),
    ))
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
    let mut gametype: Option<GameType> = None;
    let mut empty: bool = false;
    let mut full: bool = false;

    let (input, filteroptions) = separated_list0(tag(b" "), filteroption)(input)?;
    for filteroption in filteroptions {
        match filteroption {
            FilterOption::GameType(g) => {
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
    fn test_message_prefix_empty() {
        let data = &b""[..];
        let result = message_prefix::<VerboseError<_>>(data);
        assert_eq!(
            result,
            Err(nom::Err::Error(VerboseError {
                errors: vec![
                    (&b""[..], VerboseErrorKind::Nom(nom::error::ErrorKind::Tag)),
                    (
                        &b""[..],
                        VerboseErrorKind::Dpmaster(ErrorKind::MessagePrefix)
                    ),
                    (&b""[..], VerboseErrorKind::Context("message prefix")),
                ]
            }))
        );
    }

    #[test]
    fn test_message_prefix_invalid() {
        let data = &b"hurz"[..];
        let result = message_prefix::<VerboseError<_>>(data);
        assert_eq!(
            result,
            Err(nom::Err::Error(VerboseError {
                errors: vec![
                    (
                        &b"hurz"[..],
                        VerboseErrorKind::Nom(nom::error::ErrorKind::Tag)
                    ),
                    (
                        &b"hurz"[..],
                        VerboseErrorKind::Dpmaster(ErrorKind::MessagePrefix)
                    ),
                    (&b"hurz"[..], VerboseErrorKind::Context("message prefix")),
                ]
            }))
        );
    }

    #[test]
    fn test_message_prefix() {
        let data = b"\xFF\xFF\xFF\xFF";
        let result = message_prefix::<()>(data);
        assert_eq!(result, Ok((&b""[..], &b"\xFF\xFF\xFF\xFF"[..])));
    }

    #[test]
    fn test_heartbeat_message_dp() {
        let data = &b"heartbeat DarkPlaces\x0A"[..];
        let result = heartbeat(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                HeartbeatMessage::new(ProtocolName::new(b"DarkPlaces".to_vec()).unwrap(),)
            ))
        );
    }

    #[test]
    fn test_heartbeat_message_q3a() {
        let data = &b"heartbeat QuakeArena-1\x0A"[..];
        let result = heartbeat(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                HeartbeatMessage::new(ProtocolName::new(b"QuakeArena-1".to_vec()).unwrap(),)
            ))
        );
    }

    #[test]
    fn test_heartbeat_message_rtcw() {
        let data = &b"heartbeat Wolfenstein-1\x0A"[..];
        let result = heartbeat(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                HeartbeatMessage::new(ProtocolName::new(b"Wolfenstein-1".to_vec()).unwrap(),)
            ))
        );
    }

    #[test]
    fn test_heartbeat_message_woet() {
        let data = &b"heartbeat EnemyTerritory-1\x0A"[..];
        let result = heartbeat(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                HeartbeatMessage::new(ProtocolName::new(b"EnemyTerritory-1".to_vec()).unwrap(),)
            ))
        );
    }

    #[test]
    fn test_getinfo_message() {
        let data = &b"getinfo A_ch4Lleng3"[..];
        let result = getinfo(data);
        assert_eq!(
            result,
            Ok((
                &vec![][..],
                GetInfoMessage::new(Challenge::new(b"A_ch4Lleng3".to_vec()).unwrap(),)
            ))
        );
    }

    #[test]
    fn test_inforesponse_message() {
        let data = &b"infoResponse\x0A\\sv_maxclients\\8\\clients\\0"[..];
        let result = inforesponse(data);
        let mut info = Info::new();
        info.insert(
            InfoKey::new(b"sv_maxclients".to_vec()).unwrap(),
            InfoValue::new(b"8".to_vec()).unwrap(),
        );
        info.insert(
            InfoKey::new(b"clients".to_vec()).unwrap(),
            InfoValue::new(b"0".to_vec()).unwrap(),
        );
        assert_eq!(result, Ok((&vec![][..], InfoResponseMessage::new(info),)));
    }

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
                    FilterOptions::new(Some(GameType::new(b"0".to_vec()).unwrap()), true, true)
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
                    Some(GameName::new(b"Nexuiz".to_vec()).unwrap()),
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
                    Some(GameName::new(b"qfusion".to_vec()).unwrap()),
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
