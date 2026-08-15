//! A non-recursive lexical preflight bounds nesting before the Rust parser runs.

pub(super) const MAX_SYNTAX_DEPTH: usize = 256;

pub(super) fn check_syntax_depth(source: &str) -> Result<(), String> {
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut depth = 0_usize;
    let mut angle_depth = 0_usize;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"//") {
            index = skip_line_comment(bytes, index + 2);
        } else if bytes[index..].starts_with(b"/*") {
            index = skip_block_comment(bytes, index + 2)?;
        } else if let Some(end) = raw_string_end(bytes, index) {
            index = end;
        } else if bytes[index] == b'"' {
            index = skip_quoted(bytes, index + 1, b'"');
        } else if bytes[index] == b'\'' {
            index = char_literal_end(source, index).unwrap_or(index + 1);
        } else {
            match bytes[index] {
                b'(' | b'[' | b'{' => {
                    depth += 1;
                    if bytes[index] == b'{' {
                        angle_depth = 0;
                    }
                    if depth + angle_depth > MAX_SYNTAX_DEPTH {
                        return Err(format!(
                            "Rust syntax exceeds the {MAX_SYNTAX_DEPTH}-level nesting safety limit"
                        ));
                    }
                }
                b')' | b']' | b'}' => {
                    depth = depth.saturating_sub(1);
                    if bytes[index] == b'}' {
                        angle_depth = 0;
                    }
                }
                b'<' => {
                    angle_depth += 1;
                    if depth + angle_depth > MAX_SYNTAX_DEPTH {
                        return Err(format!(
                            "Rust syntax exceeds the {MAX_SYNTAX_DEPTH}-level nesting safety limit"
                        ));
                    }
                }
                b'>' => angle_depth = angle_depth.saturating_sub(1),
                b';' => angle_depth = 0,
                _ => {}
            }
            index += 1;
        }
    }
    Ok(())
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while index < bytes.len() && bytes[index] != b'\n' {
        index += 1;
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> Result<usize, String> {
    let mut depth = 1_usize;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"/*") {
            depth += 1;
            if depth > MAX_SYNTAX_DEPTH {
                return Err(format!(
                    "Rust block comments exceed the {MAX_SYNTAX_DEPTH}-level nesting safety limit"
                ));
            }
            index += 2;
        } else if bytes[index..].starts_with(b"*/") {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return Ok(index);
            }
        } else {
            index += 1;
        }
    }
    Ok(index)
}

fn raw_string_end(bytes: &[u8], index: usize) -> Option<usize> {
    if bytes.get(index) != Some(&b'r') {
        return None;
    }
    let mut opening = index + 1;
    while bytes.get(opening) == Some(&b'#') {
        opening += 1;
    }
    if bytes.get(opening) != Some(&b'"') {
        return None;
    }
    let hashes = opening - index - 1;
    let mut cursor = opening + 1;
    while cursor < bytes.len() {
        if bytes[cursor] == b'"'
            && bytes
                .get(cursor + 1..cursor + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(cursor + 1 + hashes);
        }
        cursor += 1;
    }
    Some(bytes.len())
}

fn skip_quoted(bytes: &[u8], mut index: usize, quote: u8) -> usize {
    while index < bytes.len() {
        match bytes[index] {
            b'\\' => index = (index + 2).min(bytes.len()),
            byte if byte == quote => return index + 1,
            _ => index += 1,
        }
    }
    index
}

fn char_literal_end(source: &str, index: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let start = index + 1;
    match bytes.get(start)? {
        b'\\' => escaped_char_end(bytes, start + 1),
        b'\'' | b'\n' | b'\r' | b'\t' => None,
        _ => {
            let character = source.get(start..)?.chars().next()?;
            let closing = start + character.len_utf8();
            (bytes.get(closing) == Some(&b'\'')).then_some(closing + 1)
        }
    }
}

fn escaped_char_end(bytes: &[u8], escape: usize) -> Option<usize> {
    let closing = match bytes.get(escape)? {
        b'x' => escape + 3,
        b'u' if bytes.get(escape + 1) == Some(&b'{') => {
            let mut cursor = escape + 2;
            while !matches!(bytes.get(cursor), None | Some(b'}')) {
                cursor += 1;
            }
            cursor + 1
        }
        _ => escape + 1,
    };
    (bytes.get(closing) == Some(&b'\'')).then_some(closing + 1)
}

#[cfg(test)]
#[path = "depth_test.rs"]
mod depth_test;
