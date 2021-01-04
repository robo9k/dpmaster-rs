//! deserializer for messages

use crate::messages::{FilterOptions, GetServersMessage};
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::is_digit;
use nom::combinator::opt;
use nom::multi::separated_list;
use nom::sequence::{preceded, tuple};
use nom::IResult;

fn message_prefix(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"\xFF\xFF\xFF\xFF")(input)
}

fn getservers_command(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag(b"getservers")(input)
}

fn is_space(chr: u8) -> bool {
    b' ' == chr
}

fn game_name(input: &[u8]) -> IResult<&[u8], Option<Vec<u8>>> {
    let (input, game_name) = opt(take_while1(|chr| !(is_digit(chr) || is_space(chr))))(input)?;
    Ok((input, game_name.map(|game_name| game_name.to_vec())))
}

fn protocol_number(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, protocol_bytes) = take_while(is_digit)(input)?;
    let protocol_str = std::str::from_utf8(protocol_bytes).unwrap(); // TODO
    let protocol_number = u32::from_str_radix(protocol_str, 10).unwrap(); // TODO
    Ok((input, protocol_number))
}

enum FilterOption {
    Gametype(Vec<u8>),
    Empty,
    Full,
}

fn filteroption_gametype(input: &[u8]) -> IResult<&[u8], FilterOption> {
    let (input, gametype) = preceded(tag(b"gametype="), take_while1(|chr| chr != b' '))(input)?;
    Ok((input, FilterOption::Gametype(gametype.to_vec())))
}

fn filteroption_empty(input: &[u8]) -> IResult<&[u8], FilterOption> {
    let (input, _) = tag(b"empty")(input)?;
    Ok((input, FilterOption::Empty))
}

fn filteroption_full(input: &[u8]) -> IResult<&[u8], FilterOption> {
    let (input, _) = tag(b"full")(input)?;
    Ok((input, FilterOption::Full))
}

fn filteroption(input: &[u8]) -> IResult<&[u8], FilterOption> {
    alt((filteroption_gametype, filteroption_empty, filteroption_full))(input)
}

fn filteroptions(input: &[u8]) -> IResult<&[u8], FilterOptions> {
    let mut gametype: Option<Vec<u8>> = None;
    let mut empty: bool = false;
    let mut full: bool = false;

    let (input, filteroptions) = separated_list(tag(b" "), filteroption)(input)?;
    for filteroption in &filteroptions {
        match filteroption {
            FilterOption::Gametype(ref g) => {
                gametype = Some(g.to_vec());
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

fn getservers_payload(input: &[u8]) -> IResult<&[u8], GetServersMessage> {
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

pub fn getservers(input: &[u8]) -> IResult<&[u8], GetServersMessage> {
    preceded(getservers_command, getservers_payload)(input)
}

pub fn getservers_message(input: &[u8]) -> IResult<&[u8], GetServersMessage> {
    preceded(message_prefix, getservers)(input)
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
}
