use std::ffi::OsStr;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use glob::glob;
use itertools::Itertools;

use tantivy::doc;
use tantivy::schema::*;

use crate::markdown::{
    extract_text::extract_text,
    frontmatter::{split_frontmatter_and_content, FrontMatter},
    read_string,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Lang {
    Ja,
    En,
}

impl Lang {
    pub fn to_str(&self) -> &str {
        match self {
            Lang::Ja => "ja",
            Lang::En => "en",
        }
    }

    pub fn from_str(lang: &str) -> Result<Self> {
        match lang.to_lowercase().as_str() {
            "ja" => Ok(Lang::Ja),
            "en" => Ok(Lang::En),
            _ => Err(anyhow!("Now support ja and en only!")),
        }
    }

    pub fn tokenizer_name(&self) -> String {
        "lang_".to_string() + self.to_str()
    }
}

#[derive(Debug)]
pub struct Post {
    slug: String,
    matter: FrontMatter,
    body: String,
    raw_text: String,
}

impl Post {
    pub fn lang(&self) -> &str {
        self.matter.lang()
    }

    pub fn uuid(&self) -> String {
        self.matter.id()
    }

    pub fn from_path(path: &PathBuf) -> Self {
        let slug = path
            .file_name()
            .map(rsplit_file_at_dot)
            .and_then(|(before, after)| before.or(after))
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let markdown_text = read_string(&path).unwrap();
        let (frontmatter, body) = split_frontmatter_and_content(&markdown_text);
        let raw_text = extract_text(&body);
        Self {
            slug,
            matter: frontmatter.unwrap(),
            body: body.to_string(),
            raw_text,
        }
    }
    pub fn to_doc(&self, schema: &Schema) -> Document {
        fn need_field(s: &str) -> String {
            format!("{} is need in schema", s)
        }
        let uuid = schema.get_field("uuid").expect(&need_field("uuid"));
        let slug = schema.get_field("slug").expect(&need_field("slug"));
        let title = schema.get_field("title").expect(&need_field("title"));
        let description = schema
            .get_field("description")
            .expect(&need_field("description"));
        let body = schema.get_field("body").expect(&need_field("body"));
        let lang = schema.get_field("lang").expect(&need_field("lang"));
        let category = schema.get_field("category").expect(&need_field("category"));
        let tags = schema.get_field("tags").expect(&need_field("tags"));
        let raw_text = schema.get_field("raw_text").expect(&need_field("raw_text"));

        let tag_val = match self.matter.tags() {
            Some(tags) => tags.join(" "),
            None => "".to_string(),
        };

        doc!(
            uuid => self.uuid(),
            slug => self.slug.clone(),
            title => self.matter.title(),
            description => self.matter.description(),
            body => self.body.clone(),
            lang => self.matter.lang(),
            category => self.matter.category(),
            tags => tag_val,
            raw_text => self.raw_text.clone(),
        )
    }
}

unsafe fn u8_slice_as_os_str(s: &[u8]) -> &OsStr {
    // SAFETY: see the comment of `os_str_as_u8_slice`
    &*(s as *const [u8] as *const OsStr)
}

fn os_str_as_u8_slice(s: &OsStr) -> &[u8] {
    unsafe { &*(s as *const OsStr as *const [u8]) }
}

// basic workhorse for splitting stem and extension
fn rsplit_file_at_dot(file: &OsStr) -> (Option<&OsStr>, Option<&OsStr>) {
    if os_str_as_u8_slice(file) == b".." {
        return (Some(file), None);
    }

    // The unsafety here stems from converting between &OsStr and &[u8]
    // and back. This is safe to do because (1) we only look at ASCII
    // contents of the encoding and (2) new &OsStr values are produced
    // only from ASCII-bounded slices of existing &OsStr values.
    let mut iter = os_str_as_u8_slice(file).rsplitn(2, |b| *b == b'.');
    let after = iter.next();
    let before = iter.next();
    if before == Some(b"") {
        (Some(file), None)
    } else {
        unsafe {
            (
                before.map(|s| u8_slice_as_os_str(s)),
                after.map(|s| u8_slice_as_os_str(s)),
            )
        }
    }
}

pub fn get_all_posts(glob_pattern: &str) -> Result<Vec<Post>> {
    let posts = glob(glob_pattern)?
        .into_iter()
        .filter_map(|path| path.ok())
        .map(|path| {
            // should be /path/to/filename.md
            Post::from_path(&path)
        })
        .collect_vec();

    Ok(posts)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_all_posts() {
        dbg!(&get_all_posts("./test/posts/**/*.md").unwrap());
    }
}
