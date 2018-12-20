pub fn is_atom_char(c: char) -> bool {
    !c.is_whitespace() && !c.is_control() && c != '(' && c != ')'
}

pub fn is_symbol_char(c: char) -> bool {
    is_atom_char(c) && (c.is_alphanumeric() || c == '-' || c == '_' || c == '?' || c == '*')
}

pub fn find_closing_delim(s: &str, d_plus: char, d_minus: char) -> Option<usize> {
    let mut depth = 0;

    for (idx, c) in s.chars().enumerate() {
        if d_plus == d_minus {
            if c == d_plus {
                depth = !depth;
            }
        } else {
            match c {
                x if x == d_plus => depth += 1,
                x if x == d_minus => depth -= 1,
                _ => (),
            }
        }

        if depth == 0 {
            return Some(idx);
        }
    }

    None
}
