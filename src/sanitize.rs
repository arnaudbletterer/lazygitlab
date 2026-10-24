use std::borrow::Cow;

fn is_unsafe_control(c: char) -> bool {
    matches!(
        c,
        '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1a}' | '\u{1c}'..='\u{1f}' | '\u{7f}'..='\u{9f}'
    )
}

fn is_plain_sgr(params: &str) -> bool {
    params
        .chars()
        .all(|n| n.is_ascii_digit() || n == ';' || n == ':')
}

pub fn strip_control_sequences(input: &str) -> Cow<'_, str> {
    strip(input, true)
}

pub fn strip_all_escapes(input: &str) -> Cow<'_, str> {
    strip(input, false)
}

fn strip(input: &str, keep_sgr: bool) -> Cow<'_, str> {
    if !input.chars().any(|c| c == '\u{1b}' || is_unsafe_control(c)) {
        return Cow::Borrowed(input);
    }
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            if !is_unsafe_control(c) {
                out.push(c);
            }
            continue;
        }
        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                let mut params = String::new();
                let mut final_byte = None;
                for n in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&n) {
                        final_byte = Some(n);
                        break;
                    }
                    if !('\u{20}'..='\u{3f}').contains(&n) {
                        break;
                    }
                    params.push(n);
                }
                if keep_sgr && final_byte == Some('m') && is_plain_sgr(&params) {
                    out.push_str("\u{1b}[");
                    out.push_str(&params);
                    out.push('m');
                }
            }
            Some(']' | 'P' | 'X' | '^' | '_') => {
                chars.next();
                while let Some(n) = chars.next() {
                    if n == '\u{7}' {
                        break;
                    }
                    if n == '\u{1b}' {
                        if chars.peek() == Some(&'\\') {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    Cow::Owned(out)
}
