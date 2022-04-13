enum State {
    Ready,
    InComment,
    EndComment { cur: usize },
}

pub fn remove_comments(text: &str) -> String {
    let mut s = String::new();
    let mut state = State::Ready;

    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return text.to_string();
    }

    for (i, &c) in chars.iter().enumerate() {
        match state {
            State::Ready => {
                if i < chars.len() - 4 {
                    let check_chars = &chars[i..i + 4];
                    if check_chars == ['<', '!', '-', '-'] {
                        state = State::InComment;
                    } else {
                        s.push(c);
                    }
                } else {
                    s.push(c)
                }
            }
            State::InComment => {
                if i < chars.len() - 3 {
                    let check_chars = &chars[i..i + 3];
                    if check_chars == ['-', '-', '>'] {
                        state = State::EndComment { cur: 0 };
                    }
                }
            }
            State::EndComment { cur } => {
                if cur == 1 {
                    if i < chars.len() - 1 {
                        if !chars[i + 1].is_ascii_whitespace() {
                            state = State::Ready
                        } else {
                            state = State::EndComment { cur: cur + 1 }
                        }
                    } else {
                        state = State::Ready;
                    }
                } else if cur == 2 {
                    state = State::Ready;
                } else {
                    state = State::EndComment { cur: cur + 1 }
                }
            }
        }
    }

    s
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_no_comment() {
        let text = r#"
# TEST

- AAA
- BBB"#;
        let result = remove_comments(text);
        assert_eq!(text, result.as_str());
    }

    #[test]
    fn test_oneline_comment() {
        let text = "<!-- comment -->";
        let result = remove_comments(text);
        assert!(result.is_empty());
    }

    #[test]
    fn test_multiline_comment() {
        let text = r#"
<!--
multiline
comment
-->
        "#;
        let result = remove_comments(text);
        assert!(result.trim().is_empty());
    }

    #[test]
    fn test_inline_comment() {
        let text = "in<!-- comment -->line";
        let result = remove_comments(text);
        assert_eq!(result.as_str(), "inline");
    }

    #[test]
    fn test_mix() {
        let text = r#"# TEST
<!-- comment -->

<!-- 
multiline
comment
-->

TEST
"#;
        let result = remove_comments(text);
        let expected = "# TEST\n\n\nTEST\n";
        assert_eq!(result, expected);
    }
}
