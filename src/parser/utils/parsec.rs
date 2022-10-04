use std::collections::BTreeSet;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, multispace0},
    combinator::{opt, recognize},
    error::ParseError,
    multi::{many0_count, separated_list1},
    sequence::{delimited, pair, tuple},
    AsChar, Compare, IResult, InputLength, InputTake, InputTakeAtPosition, Parser,
};

use crate::parser::SinglePriv;

pub fn parse_single_priv(input: &str) -> IResult<&str, SinglePriv> {
    tuple((identifier, opt(parse_fields)))(input).map(|(remaining, (name, fields))| {
        let cols: BTreeSet<String> = fields
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        (
            remaining,
            SinglePriv {
                name: name.into(),
                cols,
            },
        )
    })
}

/// parse "(a, b, c)" t vec!["a", "b", "c"]s
pub fn parse_fields(input: &str) -> IResult<&str, Vec<&str>> {
    delb("(", ")", separated_list1(dels(tag(",")), identifier))(input)
}

/// delimited by spaces
pub fn dels<I, O, E>(parser: impl Parser<I, O, E>) -> impl FnMut(I) -> IResult<I, O, E>
where
    E: ParseError<I>,
    I: InputTakeAtPosition,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    delimited(multispace0, parser, multispace0)
}

/// delimited by brackets
pub fn delb<I, T, O, E>(
    skip1: T,
    skip2: T,
    parser: impl Parser<I, O, E>,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    E: ParseError<I>,
    I: InputTake + InputTakeAtPosition + Compare<T>,
    T: InputLength + Clone,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    delimited(dels(tag(skip1)), parser, dels(tag(skip2)))
}

/// parse identifier
pub fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}
