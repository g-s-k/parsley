use std::str::FromStr;

use super::super::{utils, LispError};
use super::Primitive;

impl FromStr for Primitive {
    type Err = LispError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "#t" => return Ok(Primitive::Boolean(true)),
            "#f" => return Ok(Primitive::Boolean(false)),
            _ => (),
        }

        if let Ok(num) = s.parse::<f64>() {
            return Ok(Primitive::Number(num));
        }

        if s.len() == 3 && s.starts_with("#\\") {
            return Ok(Primitive::Character(s.chars().nth(2).unwrap()));
        }

        if s.starts_with('"') && s.ends_with('"') {
            match utils::find_closing_delim(s.chars(), '"', '"') {
                Some(idx) if idx + 1 == s.len() => {
                    return Ok(Primitive::String(s.get(1..idx).unwrap().to_string()));
                }
                _ => (),
            }
        }

        if s.chars().all(utils::is_symbol_char) {
            return Ok(Primitive::Symbol(s.to_string()));
        }

        Err(LispError::SyntaxError { exp: s.to_string() })
    }
}
