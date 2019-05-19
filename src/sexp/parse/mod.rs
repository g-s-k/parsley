use std::str::FromStr;

use super::SExp::{self, Atom, Null};
use super::{utils, Error, Primitive, Result};

mod tests;

#[derive(Debug, Clone, PartialEq)]
enum Token {
    OpenParen,
    OpenHashParen,
    CloseParen,
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    StringLiteral(String),
    Atom(String),
}

impl Token {
    fn from_sigil(s: &str) -> Option<Self> {
        match s {
            "(" => Some(Token::OpenParen),
            "#(" => Some(Token::OpenHashParen),
            ")" => Some(Token::CloseParen),
            "'" => Some(Token::Quote),
            "`" => Some(Token::Quasiquote),
            "," => Some(Token::Unquote),
            ",@" => Some(Token::UnquoteSplicing),
            _ => None,
        }
    }
}

impl FromStr for Token {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Some(t) = Self::from_sigil(s) {
            Ok(t)
        } else {
            if s.starts_with('"') && s.ends_with('"') {
                return Ok(Token::StringLiteral(s[1..s.len() - 1].into()));
            }

            if s.chars().all(utils::is_atom_char) {
                return Ok(Token::Atom(s.into()));
            }

            Err(s.into())
        }
    }
}

fn get_next_token(s: &str) -> std::result::Result<(Option<Token>, &str), String> {
    let mut s = s.trim_start();

    // throw out comments
    if s.starts_with(';') {
        let next_newline = s.find('\n').unwrap_or_else(|| s.len());
        s = &s[next_newline..];
    }

    s = s.trim_start();
    if s.is_empty() {
        return Ok((None, s));
    }

    // special handling for string literals
    if s.starts_with('"') {
        let mut pos = 1;
        let mut esc = false;
        for c in s[1..].chars() {
            match c {
                '\\' => esc = !esc,
                '"' if !esc => break,
                _ => esc = false,
            }
            pos += 1;
        }
        if pos == s.len() - 1 && !s.ends_with('"') {
            return Err(s.into());
        } else {
            return Ok((Some(s[..=pos].parse()?), &s[pos + 1..]));
        }
    }

    // sigils - can be 1 or 2 chars
    for len in 1..3 {
        if len <= s.len() {
            let (t, rest) = s.split_at(len);
            if let Some(tok) = Token::from_sigil(t) {
                return Ok((Some(tok), rest));
            }
        }
    }

    // atom/primitive values
    let pos = s
        .find(|c| !utils::is_atom_char(c))
        .unwrap_or_else(|| s.len());
    Ok((Some(s[..pos].parse()?), &s[pos..]))
}

fn lex(mut s: &str) -> std::result::Result<Vec<Token>, String> {
    let mut tokens = Vec::new();

    while !s.is_empty() {
        let (tok, new_s) = get_next_token(s).map_err(String::from)?;
        s = new_s;
        if let Some(tok) = tok {
            tokens.push(tok);
        }
    }

    Ok(tokens)
}

fn parse_list_tokens(tokens: &[Token]) -> std::result::Result<(Vec<SExp>, &[Token]), Error> {
    let mut idx = 1;
    let mut n = 1;

    for tok in &tokens[1..] {
        match *tok {
            Token::OpenParen | Token::OpenHashParen => n += 1,
            Token::CloseParen => n -= 1,
            _ => (),
        }

        if n == 0 {
            break;
        }
        idx += 1;
    }

    if n != 0 {
        return Err(Error::Syntax {
            exp: format!("unmatched paren(s) in {:?}", tokens),
        });
    }

    let mut list_tokens = &tokens[1..idx];
    let mut list_out = Vec::new();

    while !list_tokens.is_empty() {
        let (expr, new_list_tokens) = get_next_sexp(list_tokens)?;
        list_tokens = new_list_tokens;
        list_out.push(expr);
    }

    Ok((list_out, &tokens[idx + 1..]))
}

fn dequote(mut tokens: &[Token]) -> (Vec<SExp>, &[Token]) {
    let mut v = Vec::new();

    while !tokens.is_empty() {
        let quote = SExp::sym(match tokens[0] {
            Token::Quote => "quote",
            Token::Quasiquote => "quasiquote",
            Token::Unquote => "unquote",
            Token::UnquoteSplicing => "unquote-splicing",
            _ => break,
        });

        v.push(quote);
        tokens = &tokens[1..];
    }

    (v, tokens)
}

fn get_next_sexp(tokens: &[Token]) -> std::result::Result<(SExp, &[Token]), Error> {
    let (prefixes, tokens) = dequote(tokens);

    let mut quotable = match tokens.split_first() {
        Some((Token::Atom(s), rest)) => (Atom(s.parse()?), rest),
        Some((Token::StringLiteral(s), rest)) => (Atom(Primitive::String(s.to_string())), rest),
        Some((Token::OpenParen, rest)) => {
            if let Some((Token::CloseParen, rest)) = rest.split_first() {
                (Null, rest)
            } else {
                parse_list_tokens(tokens).map(|(v, t)| (v.into(), t))?
            }
        }
        Some((Token::OpenHashParen, _)) => {
            parse_list_tokens(tokens).map(|(v, t)| (Atom(Primitive::Vector(v)), t))?
        }
        _ => {
            return Err(Error::Syntax {
                exp: format!("{:#?}", tokens),
            })
        }
    };

    for prefix in prefixes.into_iter().rev() {
        quotable.0 = Null.cons(quotable.0).cons(prefix);
    }

    Ok(quotable)
}

impl FromStr for SExp {
    type Err = Error;

    fn from_str(s: &str) -> Result {
        let token_list = lex(s).map_err(|st| Error::Syntax { exp: st })?;
        let mut tokens = &token_list[..];

        let mut exprs = vec![Self::sym("begin")];
        while !tokens.is_empty() {
            let (expr, remaining) = get_next_sexp(tokens)?;
            tokens = remaining;
            exprs.push(expr);
        }

        // don't need `begin` expression if there's only one inside
        if exprs.len() == 2 {
            return Ok(exprs.remove(1));
        }

        Ok(exprs.into())
    }
}
