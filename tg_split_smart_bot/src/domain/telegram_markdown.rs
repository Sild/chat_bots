pub fn escape_markdown_v2(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '_' | '*' | '[' | ']' | '(' | ')' | '~' | '`' | '>' | '#' | '+' | '-' | '=' | '|'
            | '{' | '}' | '.' | '!' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::escape_markdown_v2;

    #[test]
    fn test_escape_markdown_v2_escapes_special_characters() {
        assert_eq!(
            escape_markdown_v2("Alice - [Bob]!"),
            "Alice \\- \\[Bob\\]\\!"
        );
    }
}
