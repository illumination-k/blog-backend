use std::ffi::OsStr;

use anyhow::{Result, Context};
use glob::glob;
use itertools::Itertools;

use tantivy::schema::*;
use tantivy::doc;

use crate::markdown::{
    frontmatter::{split_frontmatter_and_content, FrontMatter},
    read_string, extract_text::extract_text,
};

#[derive(Debug)]
pub struct Post {
    slug: String,
    matter: FrontMatter,
    body: String,
    raw_text: String,
}

impl Post {
    pub fn to_doc(&self, schema: Schema) -> Document {
        let slug = schema.get_field("slug").expect("id");
        let title = schema.get_field("title").unwrap();
        let body = schema.get_field("body").unwrap();
        let lang = schema.get_field("lang").unwrap();
        let category = schema.get_field("category").unwrap();
        let tags = schema.get_field("tags").unwrap();
        let raw_text = schema.get_field("raw_text").unwrap();

        doc!(
            slug => self.slug.clone(),
            title => self.matter.title(),
            body => self.body.clone(),
            lang => self.matter.lang(),
            category => self.matter.category(),
            raw_text => self.raw_text.clone()
        )
    }
}

unsafe fn u8_slice_as_os_str(s: &[u8]) -> &OsStr {
    // SAFETY: see the comment of `os_str_as_u8_slice`
    unsafe { &*(s as *const [u8] as *const OsStr) }
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
        unsafe { (before.map(|s| u8_slice_as_os_str(s)), after.map(|s| u8_slice_as_os_str(s))) }
    }
}

pub fn get_all_posts(glob_pattern: &str) -> Result<Vec<Post>> {
    let posts = glob(glob_pattern)?
        .into_iter()
        .filter_map(|path| path.ok())
        .map(|path| {
            let slug = path.file_name().map(rsplit_file_at_dot).and_then(|(before, after)| before.or(after)).unwrap().to_str().unwrap().to_string();
            let markdown_text = read_string(&path).unwrap();
            let (frontmatter, body) = split_frontmatter_and_content(&markdown_text);
            let raw_text = extract_text(&body);
            Post {
                slug,
                matter: frontmatter.unwrap(),
                body: body.to_string(),
                raw_text
            }
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
