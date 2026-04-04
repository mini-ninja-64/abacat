use chumsky::{
    Parser,
    error::Rich,
    extra,
    prelude::{just, one_of},
};
use saphyr::{LoadableYamlNode, ScanError, Yaml};
use thiserror::Error;

#[derive(Debug)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn new(red: u8, green: u8, blue: u8) -> Color {
        Color { red, green, blue }
    }
}

#[derive(Debug)]
pub struct Design {
    pub plain_text: Color,
    pub background: Color,
    pub answer_opacity: f64,
    pub identifier: Color,
    pub boolean: Color,
    pub base16: Color,
    pub base2: Color,
    pub base8: Color,
    pub base10: Color,
    pub base10_decimal: Color,
}

#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub author: String,
    pub description: String,
    pub design: Design,
}

// TODO: Contain source path to field
#[derive(Error, Debug)]
pub enum ParseError<'a> {
    #[error("Missing field")]
    Missing { field: &'a str },
    #[error("Incorrect type in field")]
    IncorrectType { field: &'a str, expected: &'a str },
    #[error("Field is incorrectly formatted")]
    IncorrectFormat { field: &'a str },
    #[error("Invalid YAML")]
    InvalidYaml(ScanError),
}

impl<'a> ParseError<'a> {
    pub fn missing(field: &'a str) -> ParseError<'a> {
        ParseError::Missing { field }
    }

    pub fn incorrect_type(field: &'a str, expected: &'a str) -> ParseError<'a> {
        ParseError::IncorrectType { field, expected }
    }
    pub fn incorrect_format(field: &'a str) -> ParseError<'a> {
        ParseError::IncorrectFormat { field }
    }
}

pub fn get_field_as_str<'a, 'b>(
    yaml: &'a Yaml<'a>,
    field: &'b str,
) -> Result<&'a str, ParseError<'b>> {
    get_field(yaml, field)?
        .as_str()
        .ok_or_else(|| ParseError::incorrect_type(field, "string"))
}

pub fn get_field_as_f64<'a, 'b>(yaml: &'a Yaml<'a>, field: &'b str) -> Result<f64, ParseError<'b>> {
    get_field(yaml, field)?
        .as_floating_point()
        .ok_or_else(|| ParseError::incorrect_type(field, "f64"))
}

pub fn get_field<'a, 'b>(
    yaml: &'a Yaml<'a>,
    field: &'b str,
) -> Result<&'a Yaml<'a>, ParseError<'b>> {
    yaml.as_mapping_get(field)
        .ok_or_else(|| ParseError::missing(field))
}

fn hex_to_u8_unchecked(c: char) -> u8 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'A' | 'a' => 10,
        'B' | 'b' => 11,
        'C' | 'c' => 12,
        'D' | 'd' => 13,
        'E' | 'e' => 14,
        'F' | 'f' => 15,
        _ => unreachable!(),
    }
}

pub fn chumsky_color<'src>() -> impl Parser<'src, &'src str, Color, extra::Err<Rich<'src, char>>> {
    let base16 = one_of("0123456789abcdefABCDEF");
    let hex_2 = base16.then(base16).map(|(b1, b2)| {
        let b1 = hex_to_u8_unchecked(b1);
        let b2 = hex_to_u8_unchecked(b2);
        let num = (b1 << 4) | b2;
        num
    });
    let hex_1 = base16.map(|b1| {
        let num = hex_to_u8_unchecked(b1);
        // println!("{} = {} [{} << 8 = {}]", b1, num, num, num.unbounded_shl(8));
        let num = (num << 4) | num;
        num
    });
    let col_3 = hex_1
        .then(hex_1.then(hex_1))
        .map(|(red, (green, blue))| Color { red, green, blue });
    let col_6 = hex_2
        .then(hex_2.then(hex_2))
        .map(|(red, (green, blue))| Color { red, green, blue });
    let x = just("#").ignore_then(col_6.or(col_3));
    x
}

pub fn parse_color<'a, 'b>(s: &'a str, field: &'b str) -> Result<Color, ParseError<'b>> {
    chumsky_color().parse(s).into_result().map_err(|e| {
        println!("{:?}", e);
        ParseError::incorrect_format(field)
    })
}

pub fn parse_yaml_doc(yaml: &Yaml) -> Result<Theme, ParseError<'static>> {
    let design = get_field(yaml, "design")?;
    Ok(Theme {
        name: get_field_as_str(yaml, "name")?.to_string(),
        author: get_field_as_str(yaml, "author")?.to_string(),
        description: get_field_as_str(yaml, "description")?.to_string(),
        design: Design {
            plain_text: parse_color(get_field_as_str(design, "plain_text")?, "plain_text")?,
            background: parse_color(get_field_as_str(design, "background")?, "background")?,
            answer_opacity: get_field_as_f64(design, "answer_opacity")?,
            identifier: parse_color(get_field_as_str(design, "identifier")?, "identifier")?,
            boolean: parse_color(get_field_as_str(design, "boolean")?, "boolean")?,
            base16: parse_color(get_field_as_str(design, "base16")?, "base16")?,
            base2: parse_color(get_field_as_str(design, "base2")?, "base2")?,
            base8: parse_color(get_field_as_str(design, "base8")?, "base8")?,
            base10: parse_color(get_field_as_str(design, "base10")?, "base10")?,
            base10_decimal: parse_color(
                get_field_as_str(design, "base10_decimal")?,
                "base10_decimal",
            )?,
        },
    })
}
pub fn parse_yaml<'a>(yaml: &'a str) -> Result<Theme, ParseError<'static>> {
    let docs = Yaml::load_from_str(yaml).map_err(|e| ParseError::InvalidYaml(e))?;
    parse_yaml_doc(&docs[0])
}
