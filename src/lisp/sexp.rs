use std::fmt;
use std::iter::FromIterator;
use std::rc::Rc;
use std::str::FromStr;

use quicli::prelude::*;

use super::as_atom::AsAtom;
use super::{utils, Context, LispError, LispResult, Primitive};

use self::SExp::*;

/// An S-Expression. Can be parsed from a string via FromStr, or constructed
/// programmatically.
///
/// # Examples
/// ```
/// use parsley::SExp;
/// let null = "()".parse::<SExp>().unwrap();
/// assert_eq!(null, SExp::Null);
/// ```
/// ```
/// use parsley::{AsAtom, SExp};
/// let parsed = "\"abcdefg\"".parse::<SExp>().unwrap();
/// assert_eq!(parsed, "abcdefg".as_atom());
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum SExp {
    Null,
    Atom(Primitive),
    Pair { head: Box<SExp>, tail: Box<SExp> },
}

impl fmt::Display for SExp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Null => write!(f, "'()",),
            Atom(a) => write!(f, "{}", a),
            Pair { box head, box tail } => {
                write!(f, "'({}", head)?;
                match tail {
                    Null => write!(f, ")"),
                    Atom(a) => write!(f, " . {})", a),
                    pair => {
                        let mut it = pair.to_owned().into_iter().peekable();
                        while let Some(element) = it.next() {
                            if it.peek().is_some() {
                                write!(f, " {}", element)?;
                            } else {
                                write!(f, " {})", element)?;
                            }
                        }
                        Ok(())
                    }
                }
            }
        }
    }
}

impl FromStr for SExp {
    type Err = LispError;

    fn from_str(s: &str) -> LispResult {
        let trimmed_str = s.trim();

        if trimmed_str.starts_with('(') {
            if let Some(idx) = utils::find_closing_delim(trimmed_str.chars(), '(', ')') {
                if idx + 1 < trimmed_str.len() {
                    let fixed_str = format!("(begin {})", trimmed_str);
                    return SExp::parse_str(&fixed_str);
                }
            }
        }

        SExp::parse_str(trimmed_str)
    }
}

/// An iterator over an S-Expression
pub struct SExpIterator {
    exp: SExp,
}

impl Iterator for SExpIterator {
    type Item = SExp;

    fn next(&mut self) -> Option<Self::Item> {
        match self.exp.to_owned() {
            Pair { box head, box tail } => {
                self.exp = tail;
                Some(head)
            }
            _ => None,
        }
    }
}

impl IntoIterator for SExp {
    type Item = SExp;
    type IntoIter = SExpIterator;

    fn into_iter(self) -> Self::IntoIter {
        SExpIterator { exp: self }
    }
}

impl FromIterator<SExp> for SExp {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = SExp>,
    {
        let mut exp_out = Null;

        for item in iter {
            exp_out = exp_out.cons(item);
        }

        exp_out
    }
}

impl SExp {
    fn parse_str(s: &str) -> LispResult {
        let code = s.trim();

        if code.starts_with('\'')
            && code.len() > 1
            && code.chars().skip(1).all(utils::is_symbol_char)
        {
            debug!("Matched quoted symbol: {}", code);
            Ok(Null
                .cons(Atom(code[1..].parse::<Primitive>()?))
                .cons(SExp::make_symbol("quote")))
        } else if code.chars().all(utils::is_atom_char) {
            debug!("Matched atom: {}", code);
            Ok(Atom(code.parse::<Primitive>()?))
        } else if code.starts_with("'(") && code.ends_with(')') {
            let tail = box if code.len() == 3 {
                Null.cons(Null)
            } else {
                SExp::parse_str(&code[1..])?
            };
            Ok(tail.cons(SExp::make_symbol("quote")))
        } else if code.starts_with('(') && code.ends_with(')') {
            match utils::find_closing_delim(code.chars(), '(', ')') {
                Some(idx) if idx == 1 => Ok(Null),
                Some(idx) => {
                    debug!("Matched list with length {} chars", idx + 1);
                    let mut list_str = code[1..idx].trim();
                    let mut list_out = Null;

                    while !list_str.is_empty() {
                        debug!(
                            "Processing list string with length {} chars",
                            list_str.len()
                        );

                        if list_str.ends_with(')') {
                            match utils::find_closing_delim(list_str.chars().rev(), ')', '(') {
                                None => {
                                    return Err(LispError::SyntaxError {
                                        exp: list_str.to_string(),
                                    });
                                }
                                // Some(idx2) if idx2 + 1 == list_str.len() => {
                                //     debug!("Whole string is a single list");
                                //     list_out = SExp::Pair {
                                //         head: box SExp::parse_str(list_str)?,
                                //         tail: box list_out,
                                //     };
                                //     break;
                                // }
                                Some(idx2) => {
                                    debug!("Matched sub-list with length {} chars", idx2 + 1);
                                    let mut new_idx = list_str.len() - 1 - idx2;
                                    if new_idx > 0 {
                                        if let Some('\'') = list_str.chars().nth(new_idx - 1) {
                                            new_idx -= 1;
                                        }
                                    }

                                    let (before, after) = list_str.split_at(new_idx);
                                    list_str = before.trim();
                                    list_out = Pair {
                                        head: box SExp::parse_str(after)?,
                                        tail: box list_out,
                                    };
                                }
                            }
                        } else {
                            // if let Ok(prim_val) = list_str.parse::<Primitive>() {
                            //     list_out = SExp::Pair {
                            //         head: box SExp::Atom(prim_val),
                            //         tail: box list_out,
                            //     };
                            //     break;
                            // }

                            if list_str.ends_with('"') {
                                match list_str.chars().rev().skip(1).position(|e| e == '"') {
                                    Some(idx2) => {
                                        debug!("Matched string with length {} chars", idx2);
                                        let (rest, last) =
                                            list_str.split_at(list_str.len() - 2 - idx2);
                                        list_out = list_out.cons(Atom(last.parse::<Primitive>()?));
                                        list_str = rest.trim();
                                    }
                                    None => {
                                        return Err(LispError::SyntaxError {
                                            exp: list_str.to_string(),
                                        });
                                    }
                                }
                            }

                            match list_str.chars().rev().position(|c| !utils::is_atom_char(c)) {
                                Some(idx3) => {
                                    debug!(
                                        "Matched atom in first position with length {} chars",
                                        idx3
                                    );
                                    let (rest, last) = list_str.split_at(list_str.len() - idx3);
                                    list_out = Pair {
                                        head: box SExp::parse_str(last)?,
                                        tail: box list_out,
                                    };
                                    list_str = rest.trim();
                                }
                                _ => {
                                    debug!("Entire string is an atom.");
                                    list_out = Pair {
                                        head: box SExp::parse_str(list_str)?,
                                        tail: box list_out,
                                    };
                                    break;
                                }
                            }
                        }
                    }

                    Ok(list_out)
                }
                None => Err(LispError::SyntaxError {
                    exp: code.to_string(),
                }),
            }
        } else {
            let prim = code.parse::<Primitive>()?;
            Ok(Atom(prim))
        }
    }

    /// Evaluate an S-Expression in a context.
    ///
    /// The context will retain any definitions bound during evaluation
    /// (e.g. `define`, `set!`).
    ///
    /// # Examples
    /// ```
    /// use parsley::{AsAtom, Context};
    /// use parsley::SExp::{self, Null};
    ///
    /// let exp = Null.cons(1.0.as_atom())
    ///     .cons(0.0.as_atom())
    ///     .cons(SExp::make_symbol("eq?"));
    /// let mut ctx = Context::base();
    /// let result = exp.eval(&mut ctx);
    /// assert_eq!(result.unwrap(), false.as_atom());
    /// ```
    /// ```
    /// use parsley::{AsAtom, Context};
    /// use parsley::SExp::{self, Null};
    ///
    /// let exp1 = Null.cons(10.0.as_atom())
    ///     .cons(SExp::make_symbol("x"))
    ///     .cons(SExp::make_symbol("define"));
    /// let exp2 = SExp::make_symbol("x");
    ///
    /// let mut ctx = Context::base();
    /// exp1.eval(&mut ctx);
    /// let result = exp2.eval(&mut ctx);
    /// assert_eq!(result.unwrap(), 10.0.as_atom());
    /// ```
    pub fn eval(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NullList),
            Atom(Primitive::Symbol(sym)) => match ctx.get(&sym) {
                None => Err(LispError::UndefinedSymbol { sym }),
                Some(exp) => exp.eval(ctx),
            },
            Atom(_) => Ok(self),
            Pair { box head, box tail } => {
                // handle special functions
                let new_pair = tail.to_owned().cons(head.to_owned());
                match new_pair.clone().eval_special_form(ctx) {
                    Some(result) => {
                        debug!("Special form finished evaluating.");
                        result
                    }
                    None => {
                        // handle everything else
                        debug!("Evaluating normal list: {}", new_pair);
                        let evaluated = tail
                            .into_iter()
                            .map(|e| e.eval(ctx))
                            .collect::<Result<Vec<_>, LispError>>()?
                            .into_iter()
                            .rev()
                            .collect::<SExp>()
                            .cons(head.eval(ctx)?);

                        trace!("Applying operation: {}", evaluated);
                        evaluated.apply(ctx)
                    }
                }
            }
        }
    }

    fn eval_special_form(self, ctx: &mut Context) -> Option<LispResult> {
        match self {
            Null => None,
            Atom(_) => None,
            Pair { box head, box tail } => match head {
                Atom(Primitive::Symbol(sym)) => match sym.as_ref() {
                    "lambda" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "lambda".to_string(),
                        })),
                        Atom(a) => Some(Err(LispError::NotAList {
                            atom: a.to_string(),
                        })),
                        Pair {
                            head: box params,
                            tail: box fn_body,
                        } => {
                            debug!("Creating procedure.");
                            Some(Ok(Atom(Primitive::Procedure(Rc::new(move |args| {
                                debug!("Formal parameters: {}", params);
                                let bound_params = params
                                    .to_owned()
                                    .into_iter()
                                    .zip(args.into_iter())
                                    .map(|(p, a)| Null.cons(a).cons(p))
                                    .collect();
                                debug!("Bound parameters: {}", bound_params);
                                Ok(fn_body
                                    .to_owned()
                                    .cons(bound_params)
                                    .cons(SExp::make_symbol("let")))
                            })))))
                        }
                    },
                    "define" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "define".to_string(),
                        })),
                        Atom(a) => Some(Err(LispError::NotAList {
                            atom: a.to_string(),
                        })),
                        Pair {
                            head: box head2,
                            tail:
                                box Pair {
                                    head: box defn,
                                    tail: box Null,
                                },
                        } => match head2 {
                            Atom(Primitive::Symbol(sym)) => {
                                debug!("Defining a quanitity with symbol {}", &sym);
                                ctx.define(&sym, defn.clone());
                                Some(Ok(defn))
                            }
                            Pair {
                                head: box Atom(Primitive::Symbol(sym)),
                                tail: box fn_params,
                            } => {
                                debug!("Defining a function with \"define\" syntax.");
                                ctx.define(
                                    &sym,
                                    Null.cons(defn)
                                        .cons(fn_params)
                                        .cons(SExp::make_symbol("lambda")),
                                );
                                Some(Ok(Atom(Primitive::Undefined)))
                            }
                            exp => Some(Err(LispError::SyntaxError {
                                exp: exp.to_string(),
                            })),
                        },
                        exp => Some(Err(LispError::SyntaxError {
                            exp: exp.to_string(),
                        })),
                    },
                    "set!" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "set!".to_string(),
                        })),
                        Pair {
                            head: box Atom(Primitive::Symbol(sym)),
                            tail: box defn,
                        } => Some(ctx.set(&sym, defn)),
                        exp => Some(Err(LispError::SyntaxError {
                            exp: exp.to_string(),
                        })),
                    },
                    "let" => match tail {
                        Null => Some(Err(LispError::NoArgumentsProvided {
                            symbol: "let".to_string(),
                        })),
                        Pair {
                            head: box defn_list,
                            tail: box statements,
                        } => {
                            debug!("Creating a local binding.");
                            ctx.push();

                            for defn in defn_list {
                                match defn {
                                    Pair {
                                        head: box Atom(Primitive::Symbol(key)),
                                        tail:
                                            box Pair {
                                                head: box val,
                                                tail: box Null,
                                            },
                                    } => match val.eval(ctx) {
                                        Ok(result) => ctx.define(&key, result),
                                        err => return Some(err),
                                    },
                                    exp => {
                                        return Some(Err(LispError::SyntaxError {
                                            exp: exp.to_string(),
                                        }));
                                    }
                                }
                            }

                            let mut result = Err(LispError::NullList);

                            for statement in statements {
                                result = statement.eval(ctx);

                                if result.is_err() {
                                    break;
                                }
                            }

                            ctx.pop();

                            Some(result)
                        }
                        exp => Some(Err(LispError::SyntaxError {
                            exp: exp.to_string(),
                        })),
                    },
                    "cond" => {
                        debug!("Evaluating conditional form.");
                        let else_ = SExp::make_symbol("else");

                        for case in tail {
                            match case {
                                Pair {
                                    head: box predicate,
                                    tail:
                                        box Pair {
                                            head: box consequent,
                                            tail: box Null,
                                        },
                                } => {
                                    // TODO: check if `else` clause is actually last
                                    if predicate == else_ {
                                        return Some(consequent.eval(ctx));
                                    }

                                    match predicate.eval(ctx) {
                                        Ok(Atom(Primitive::Boolean(false))) => {
                                            continue;
                                        }
                                        Ok(_) => return Some(consequent.eval(ctx)),
                                        err => return Some(err),
                                    }
                                }
                                exp => {
                                    return Some(Err(LispError::SyntaxError {
                                        exp: exp.to_string(),
                                    }));
                                }
                            }
                        }

                        // falls through if no valid predicates found
                        Some(Ok(Atom(Primitive::Void)))
                    }
                    "and" => Some(tail.eval_and(ctx)),
                    "begin" => Some(tail.eval_begin(ctx)),
                    "if" => Some(tail.eval_if(ctx)),
                    "or" => Some(tail.eval_or(ctx)),
                    "quote" => Some(Ok(tail.eval_quote())),
                    _ => None,
                },
                _ => None,
            },
        }
    }

    fn eval_begin(self, ctx: &mut Context) -> LispResult {
        match self {
            Null => Err(LispError::NoArgumentsProvided {
                symbol: "begin".to_string(),
            }),
            _ => {
                debug!("Evaluating \"begin\" sequence.");
                match self.into_iter().map(|e| e.eval(ctx)).last() {
                    Some(stuff) => stuff,
                    None => Err(LispError::SyntaxError {
                        exp: "something bad happened, idk".to_string(),
                    }),
                }
            }
        }
    }

    fn eval_and(self, ctx: &mut Context) -> LispResult {
        debug!("Evaluating 'and' expression.");
        let mut state = true.as_atom();

        for element in self {
            state = element.eval(ctx)?;

            if let Atom(Primitive::Boolean(false)) = state {
                break;
            }
        }

        Ok(state)
    }

    fn eval_or(self, ctx: &mut Context) -> LispResult {
        debug!("Evaluating 'or' expression.");
        for element in self {
            match element.eval(ctx)? {
                Atom(Primitive::Boolean(false)) => (),
                exp => {
                    return Ok(exp);
                }
            }
        }

        Ok(false.as_atom())
    }

    fn eval_if(self, ctx: &mut Context) -> LispResult {
        match self {
            Pair {
                head: box condition,
                tail:
                    box Pair {
                        head: box if_true,
                        tail:
                            box Pair {
                                head: box if_false,
                                tail: box Null,
                            },
                    },
            } => {
                debug!("Evaluating 'if' expression.");
                (match condition.eval(ctx)? {
                    Atom(Primitive::Boolean(false)) => if_false,
                    _ => if_true,
                })
                .eval(ctx)
            }
            exp => Err(LispError::SyntaxError {
                exp: exp.to_string(),
            }),
        }
    }

    fn eval_quote(self) -> Self {
        trace!("Evaluating 'quote' expression: {}", self);
        match self {
            Pair {
                box head,
                tail: box Null,
            } => head,
            _ => self,
        }
    }

    fn apply(self, ctx: &mut Context) -> LispResult {
        match self {
            Null | Atom(_) => Ok(self),
            Pair { box head, box tail } => match head {
                Atom(Primitive::Procedure(proc)) => {
                    trace!("Applying a procedure to the arguments {}", tail);
                    proc(tail)?.eval(ctx)
                }
                Atom(Primitive::Symbol(sym)) => Err(LispError::NotAProcedure {
                    exp: sym.to_string(),
                }),
                Pair {
                    head: box proc,
                    tail: box tail2,
                } => tail2.cons(proc.eval(ctx)?).eval(ctx),
                _ => Ok(tail.cons(head)),
            },
        }
    }

    pub(super) fn car(&self) -> LispResult {
        match self {
            Null => Err(LispError::NullList),
            Atom(_) => Err(LispError::NotAList {
                atom: self.to_string(),
            }),
            Pair { box head, .. } => Ok(head.to_owned()),
        }
    }

    pub(super) fn cdr(&self) -> LispResult {
        match self {
            Null => Err(LispError::NullList),
            Atom(_) => Err(LispError::NotAList {
                atom: self.to_string(),
            }),
            Pair { box tail, .. } => Ok(tail.to_owned()),
        }
    }

    pub fn cons(self, exp: Self) -> Self {
        Pair {
            head: box exp,
            tail: box self,
        }
    }

    /// Convenience method to build a symbolic atom.
    ///
    /// # Example
    /// ```
    /// use parsley::{Context, SExp};
    /// let mut ctx = Context::base();
    ///
    /// // A null list is an empty application
    /// assert!(SExp::Null.eval(&mut ctx).is_err());
    ///
    /// // The symbol `null` (defined in `Context::base`) creates a null list
    /// let result = SExp::make_symbol("null").eval(&mut ctx);
    /// assert!(result.is_ok());
    /// assert_eq!(result.unwrap(), SExp::Null);
    /// ```
    pub fn make_symbol(sym: &str) -> Self {
        Atom(Primitive::Symbol(sym.to_string()))
    }
}
