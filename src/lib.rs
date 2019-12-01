#[macro_use]
extern crate pest_derive;

use pest::{error::Error, Parser};

#[derive(Parser)]
#[grammar = "spec.pest"]
struct IdentParser;

#[derive(PartialEq, Debug)]
pub enum Wrapper {
    TripleCurly,
    DoubleCurly,
    Curly,
    DollarCurly,
    CurlyHash,
    CurlyPercent,
}

#[derive(PartialEq, Debug)]
pub struct Item<'a> {
    pub wrapper: Wrapper,
    pub text: &'a str,
}

impl<'a> Item<'a> {
    fn new(wrapper: Wrapper, text: &'a str) -> Item<'a> {
        Item { wrapper, text }
    }
}

#[derive(PartialEq, Debug)]
pub enum Element<'a> {
    Text(&'a str),
    Wrapped(Item<'a>),
}

pub fn parse<'e>(s: &'e str) -> Result<Vec<Element<'e>>, Error<Rule>> {
    let pairs = IdentParser::parse(Rule::expression, s)?;

    let result: Vec<Element<'e>> = pairs
        .take_while(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| match pair.as_rule() {
            Rule::text => Element::Text(pair.into_inner().next().unwrap().as_str()),
            Rule::triple_curly => Element::Wrapped(Item::new(
                Wrapper::TripleCurly,
                pair.into_inner().next().unwrap().as_str(),
            )),
            Rule::double_curly => Element::Wrapped(Item::new(
                Wrapper::DoubleCurly,
                pair.into_inner().next().unwrap().as_str(),
            )),
            Rule::curly => Element::Wrapped(Item::new(
                Wrapper::Curly,
                pair.into_inner().next().unwrap().as_str(),
            )),
            Rule::dollar_curly => Element::Wrapped(Item::new(
                Wrapper::DollarCurly,
                pair.into_inner().next().unwrap().as_str(),
            )),
            Rule::curly_hash => Element::Wrapped(Item::new(
                Wrapper::CurlyHash,
                pair.into_inner().next().unwrap().as_str(),
            )),
            Rule::curly_percent => Element::Wrapped(Item::new(
                Wrapper::CurlyPercent,
                pair.into_inner().next().unwrap().as_str(),
            )),
            Rule::EOI => Element::Text(""),
            _ => unreachable!(),
        })
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_curly() {
        assert_eq!(
            parse("Hello, {name}!").unwrap(),
            vec![
                Element::Text("Hello, "),
                Element::Wrapped(Item::new(Wrapper::Curly, "name")),
                Element::Text("!")
            ]
        );
    }

    #[test]
    fn parse_double_curly() {
        assert_eq!(
            parse("Hello, {{ user.name }}").unwrap(),
            vec![
                Element::Text("Hello, "),
                Element::Wrapped(Item::new(Wrapper::DoubleCurly, " user.name "))
            ]
        );
    }

    #[test]
    fn parse_dollar_curly() {
        assert_eq!(
            parse("A ${plus} B").unwrap(),
            vec![
                Element::Text("A "),
                Element::Wrapped(Item::new(Wrapper::DollarCurly, "plus")),
                Element::Text(" B"),
            ]
        );
    }

    #[test]
    fn parse_curly_hash() {
        assert_eq!(
            parse("{##}").unwrap(),
            vec![Element::Wrapped(Item::new(Wrapper::CurlyHash, ""))]
        );

        assert_eq!(
            parse("{#}").unwrap(),
            vec![Element::Wrapped(Item::new(Wrapper::Curly, "#")),]
        );

        assert_eq!(
            parse("{# comment #}").unwrap(),
            vec![Element::Wrapped(Item::new(Wrapper::CurlyHash, " comment "))]
        );
    }

    #[test]
    fn parse_curly_percent() {
        assert_eq!(
            parse("{%}").unwrap(),
            vec![Element::Wrapped(Item::new(Wrapper::Curly, "%"))]
        );

        assert_eq!(
            parse("Hello, {% name %}").unwrap(),
            vec![
                Element::Text("Hello, "),
                Element::Wrapped(Item::new(Wrapper::CurlyPercent, " name "))
            ]
        );
    }

    #[test]
    fn parse_short() {
        assert_eq!(
            parse("{n}").unwrap(),
            vec![Element::Wrapped(Item::new(Wrapper::Curly, "n"))]
        );

        assert_eq!(
            parse("{}").unwrap(),
            vec![Element::Wrapped(Item::new(Wrapper::Curly, ""))]
        );
    }

    #[test]
    fn parse_broken() {
        assert!(parse("${").is_err());
        assert!(parse("{{todo..").is_err());
        assert!(parse("broken {%").is_err());
    }
}
