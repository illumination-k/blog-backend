use std::ffi::OsStr;
use std::path::Path;

use anyhow::{anyhow, Result};
use glob::glob;
use itertools::Itertools;

use tantivy::schema::*;
use tantivy::{doc, DateTime};

use crate::markdown::{
    extract_text::extract_text,
    frontmatter::{split_frontmatter_and_content, FrontMatter},
};

use crate::io::read_string;

fn need_field(s: &str) -> String {
    format!("{} is need in schema", s)
}

pub fn get_field(field_name: &str, schema: &Schema) -> Field {
    schema
        .get_field(field_name)
        .unwrap_or_else(|| panic!("{}", need_field(field_name)))
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lang {
    Ja,
    En,
}

impl Lang {
    pub fn as_str(&self) -> &str {
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
        "lang_".to_string() + self.as_str()
    }
}

#[derive(Debug, PartialEq)]
pub struct Post {
    slug: String,
    matter: FrontMatter,
    body: String,
    raw_text: String,
}

impl Post {
    pub fn slug(&self) -> String {
        self.slug.clone()
    }

    #[allow(dead_code)]
    pub fn lang(&self) -> Lang {
        self.matter.lang()
    }

    pub fn uuid(&self) -> String {
        self.matter.uuid()
    }

    #[allow(dead_code)]
    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn title(&self) -> String {
        self.matter.title()
    }

    pub fn matter(&self) -> FrontMatter {
        self.matter.to_owned()
    }

    /// **CAUSION!**  
    /// This function do not return strict equal.
    /// If updated_at and created_at in `self.matter` is `None`,
    /// this function do not compare updated_at and created_at.  
    /// It is useful when comparing the post from doc and
    /// the post which has no updated_at and created_at field.  
    pub fn equal_from_doc(&self, other: &Self) -> bool {
        self.body == other.body
            && self.slug == other.slug
            && self.raw_text == other.raw_text
            && self.matter.equal_matter_from_doc(&other.matter)
    }

    pub fn from_path(path: &Path) -> Self {
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
        let raw_text = extract_text(body);
        Self {
            slug,
            matter: frontmatter.unwrap(),
            body: body.to_string(),
            raw_text,
        }
    }

    pub fn from_doc(doc: &Document, schema: &Schema) -> Self {
        fn get_text(doc: &Document, field_name: &str, schema: &Schema) -> String {
            let field = get_field(field_name, schema);
            doc.get_first(field).unwrap().text().unwrap().to_string()
        }

        fn get_date(doc: &Document, field_name: &str, schema: &Schema) -> DateTime {
            let field = get_field(field_name, schema);
            doc.get_first(field)
                .unwrap()
                .date_value()
                .unwrap()
                .to_owned()
        }

        let uuid = get_text(doc, "uuid", schema);
        let slug = get_text(doc, "slug", schema);
        let title = get_text(doc, "title", schema);
        let description = get_text(doc, "description", schema);
        let body = get_text(doc, "body", schema);
        let lang = get_text(doc, "lang", schema);
        let category = get_text(doc, "category", schema);
        let tags = get_text(doc, "tags", schema);
        let raw_text = get_text(doc, "raw_text", schema);
        let created_at = get_date(doc, "created_at", schema);
        let updated_at = get_date(doc, "updated_at", schema);

        let tags = if tags.is_empty() {
            None
        } else {
            Some(
                tags.split(' ')
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect_vec(),
            )
        };

        Self {
            slug,
            body,
            raw_text,
            matter: FrontMatter::new(
                uuid,
                title,
                description,
                category,
                Lang::from_str(&lang).unwrap(),
                tags,
                Some(created_at),
                Some(updated_at),
            ),
        }
    }

    pub fn to_doc(
        &self,
        schema: &Schema,
        created_at: &DateTime,
        updated_at: &DateTime,
    ) -> Document {
        let uuid = get_field("uuid", schema);
        let slug = get_field("slug", schema);
        let title = get_field("title", schema);
        let description = get_field("description", schema);
        let body = get_field("body", schema);
        let lang = get_field("lang", schema);
        let category = get_field("category", schema);
        let tags = get_field("tags", schema);
        let raw_text = get_field("raw_text", schema);

        let tag_val = match self.matter.tags() {
            Some(tags) => tags.join(" "),
            None => "".to_string(),
        };

        let mut doc = doc!(
            uuid => self.uuid(),
            slug => self.slug.clone(),
            title => self.matter.title(),
            description => self.matter.description(),
            body => self.body.clone(),
            lang => self.matter.lang().as_str(),
            category => self.matter.category(),
            tags => tag_val,
            raw_text => self.raw_text.clone(),
        );

        doc.add_date(get_field("created_at", schema), created_at);
        doc.add_date(get_field("updated_at", schema), updated_at);

        doc
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
