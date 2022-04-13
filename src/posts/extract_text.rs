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

fn extract_inner_text(html: &mut BufferQueue) -> Vec<String> {
    let mut s = vec![];

    let mut tokenizer = Tokenizer::new(Sink(Vec::new()), Default::default());
    let _ = tokenizer.feed(html);
    tokenizer.end();

    for token in tokenizer.sink.0.iter() {
        match token {
            CharacterTokens(tendri) => {
                let inner = tendri.to_string().trim().to_string();
                if inner.is_empty() {
                    continue;
                }
                s.push(inner);
            }
            _ => {}
        }
    }

    s
}

pub fn extract_text(markdown_text: &str) -> String {
    let mut state = State::Ready;
    let mut s = vec![];
    let mut html_values = BufferQueue::new();

    let parser = Parser::new_ext(markdown_text, Options::empty());
    //let html_parser = parse_fragment(TokenSink, ParseOpts::default(), context_name, context_attrs)
    for e in parser {
        match e {
            Event::Text(text) => {
                if state == State::InHtml {
                    let inner_values = extract_inner_text(&mut html_values);
                    s.extend(inner_values.into_iter());
                    state = State::Ready;
                }

                s.push(text.to_string());
            }
            Event::Html(html) => {
                if state == State::Ready {
                    state = State::InHtml;
                }
                let chunk =
                    unsafe { ByteTendril::from_byte_slice_without_validating(html.as_bytes()) };
                html_values.push_back(chunk.try_reinterpret().unwrap());
            }
            _ => continue,
        }
    }

    s.join("\n")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::read_string;
    use crate::posts::frontmatter::split_frontmatter_and_content;

    #[test]
    fn test_extract_text() {
        let markdown_text = read_string(&"test/posts/ja/c1/test_post.md".to_string()).unwrap();
        let (_, markdown_text) = split_frontmatter_and_content(&markdown_text);

        let text = extract_text(&markdown_text);
        assert_eq!(
            &text,
            r#"TEST
これはテストです。
リスト1
リスト2
self
fn main() {
    println!("Hello World")
}

Some Codes"#
        )
    }

    #[test]
    fn test_ignore_comment() {
        let markdown_text = r#"
<!-- comments -->

<!--
multi line
comments
-->

<div><p>inner</p></div>

## TEST
"#;
        let text = extract_text(markdown_text);
        let expected = "inner\nTEST";
        assert_eq!(text, expected);
    }
}
