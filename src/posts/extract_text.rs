use anyhow::{anyhow, Result};
use html5ever::{
    buffer_queue::BufferQueue,
    tendril::ByteTendril,
    tokenizer::{Token, TokenSink, TokenSinkResult, Tokenizer},
};
use pulldown_cmark::{Event, Options, Parser};
use Token::CharacterTokens;

#[derive(Debug, PartialEq)]
enum State {
    Ready,
    InHtml,
}

#[derive(Debug)]
struct Sink(Vec<Token>);

impl TokenSink for Sink {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_number: u64) -> TokenSinkResult<()> {
        self.0.push(token);
        TokenSinkResult::Continue
    }
}

fn extract_inner_text(html_values: &[String]) -> Result<Vec<String>> {
    let mut html = BufferQueue::new();
    for value in html_values.iter() {
        let chunk = unsafe { ByteTendril::from_byte_slice_without_validating(value.as_bytes()) };

        let buf = match chunk.try_reinterpret() {
            Ok(utf8) => utf8,
            Err(_bytes) => return Err(anyhow!(format!("Invalid html token: {}", value))),
        };
        html.push_back(buf);
    }
    let mut s = vec![];

    let mut tokenizer = Tokenizer::new(Sink(Vec::new()), Default::default());
    let _ = tokenizer.feed(&mut html);
    tokenizer.end();

    for token in tokenizer.sink.0.iter() {
        if let CharacterTokens(tendri) = token {
            let inner = tendri.to_string().trim().to_string();
            if inner.is_empty() {
                continue;
            }
            s.push(inner);
        }
    }

    Ok(s)
}

pub fn extract_text(markdown_text: &str) -> Result<String> {
    let mut state = State::Ready;
    let mut s = Vec::new();
    let mut html_values = Vec::new();

    let parser = Parser::new_ext(markdown_text, Options::empty());
    //let html_parser = parse_fragment(TokenSink, ParseOpts::default(), context_name, context_attrs)
    for e in parser {
        match e {
            Event::Text(text) => {
                if state == State::InHtml {
                    let inner_values = extract_inner_text(&html_values)?;
                    s.extend(inner_values.into_iter());
                    html_values = Vec::new();
                    state = State::Ready;
                }

                s.push(text.to_string());
            }
            Event::Html(html) => {
                if state == State::Ready {
                    state = State::InHtml;
                }
                html_values.push(html.to_string())
            }
            _ => continue,
        }
    }

    if !html_values.is_empty() {
        let inner_values = extract_inner_text(&html_values)?;
        s.extend(inner_values.into_iter());
    }

    Ok(s.join("\n"))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_extract_text() -> Result<()> {
        let markdown_text = r#"
## TEST

これはテストです。

- リスト1
- リスト2

[self](test_post.md)

```rust
fn main() {
    println!("Hello World")
}
```

<summary>

<details>RUN</details>

Some Codes

</summary>
        "#;

        let text = extract_text(&markdown_text)?;
        let expected = "TEST\nこれはテストです。\nリスト1\nリスト2\nself\nfn main() {\n    println!(\"Hello World\")\n}\n\nRUN\nSome Codes";
        assert_eq!(&text, expected);
        Ok(())
    }

    #[test]
    fn test_extract_inner_text() -> Result<()> {
        let text = r#"
<summary>
<detail>detail</detail>
summary
</summary>
        "#;
        let text: Vec<String> = text.split("\n").map(|s| s.to_string()).collect();
        let extracted = extract_inner_text(&text)?;
        assert_eq!(extracted.join("\n"), "detail\nsummary");
        Ok(())
    }

    #[test]
    fn text_only_html() -> Result<()> {
        let text = r#"
<summary>
<detail>detail</detail>
summary
</summary>
        "#;

        let extracted = extract_text(&text)?;
        assert_eq!(extracted, "detail\nsummary");
        Ok(())
    }

    #[test]
    fn test_ignore_comment() -> Result<()> {
        let markdown_text = r#"
<!-- comments -->

<!--
multi line
comments
-->

<div><p>inner</p></div>

## TEST
"#;
        let text = extract_text(markdown_text)?;
        let expected = "inner\nTEST";
        assert_eq!(text, expected);
        Ok(())
    }
}
