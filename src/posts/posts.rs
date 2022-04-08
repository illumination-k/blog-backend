use std::ffi::OsStr;
use std::path::Path;

use anyhow::{anyhow, Result};
use itertools::Itertools;

use tantivy::schema::*;

use super::extract_text::extract_text;
use super::frontmatter::{split_frontmatter_and_content, FrontMatter};

use crate::datetime::{DateTimeFormat, DateTimeWithFormat};
use crate::io::read_string;
use crate::text_engine::schema::{FieldGetter, PostField};

#[cfg(test)]
use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(EnumIter, EnumCount))]
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

impl ToString for Lang {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub fn path_to_slug(path: &Path) -> String {
    path.file_name()
        .map(rsplit_file_at_dot)
        .and_then(|(before, after)| before.or(after))
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[derive(Debug, PartialEq)]
pub struct Post {
    slug: String,
    matter: FrontMatter,
    body: String,
    raw_text: Option<String>,
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

    #[allow(dead_code)]
    pub fn body_mut(&mut self) -> &mut String {
        &mut self.body
    }

    #[allow(dead_code)]
    pub fn category(&self) -> String {
        self.matter.category()
    }

    #[allow(dead_code)]
    pub fn tags(&self) -> Option<Vec<String>> {
        self.matter.tags()
    }

    pub fn title(&self) -> String {
        self.matter.title()
    }

    pub fn matter(&self) -> FrontMatter {
        self.matter.to_owned()
    }

    pub fn created_at(&self) -> Option<DateTimeWithFormat> {
        self.matter.created_at()
    }

    pub fn updated_at(&self) -> Option<DateTimeWithFormat> {
        self.matter.updated_at()
    }

    pub fn raw_text(&self) -> Option<String> {
        self.raw_text.clone()
    }

    #[allow(dead_code)]
    pub fn diff(&self, other: &Self) {
        if self.body != other.body {
            eprintln!("body: self: {} other: {}", self.body, other.body);
        }

        if self.slug != other.slug {
            eprintln!("slug: self: {} other: {}", self.slug, other.slug);
        }

        if self.raw_text != other.raw_text {
            eprintln!(
                "rawtext:\n\t self: \t{:?}\n \tother: \t{:?}\n",
                self.raw_text, other.raw_text
            );
        }

        if !self.matter.equal_matter_from_doc(&other.matter) {
            eprintln!(
                "matter:\n self: {:?}\n other: {:?}",
                self.matter, other.matter
            )
        }
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
            && self.matter.equal_matter_from_doc(&other.matter)
    }

    pub fn new(slug: String, matter: FrontMatter, body: String) -> Self {
        let raw_text = extract_text(&body);
        Self {
            slug,
            matter,
            body,
            raw_text: Some(raw_text),
        }
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let slug = path_to_slug(path);

        let markdown_text = read_string(&path).unwrap();
        let (frontmatter, body) = split_frontmatter_and_content(&markdown_text);
        let raw_text = extract_text(body);
        Ok(Self {
            slug,
            matter: frontmatter.unwrap_or_else(|| panic!("{:?} does not have frontmatter.", path)),
            body: body.to_string(),
            raw_text: Some(raw_text),
        })
    }

    pub fn from_doc(doc: &Document, schema: &Schema) -> Result<Self> {
        let fb = FieldGetter::new(schema);

        let uuid = fb.get_text(doc, PostField::Uuid)?;
        let slug = fb.get_text(doc, PostField::Slug)?;
        let title = fb.get_text(doc, PostField::Title)?;
        let description = fb.get_text(doc, PostField::Description)?;
        let body = fb.get_text(doc, PostField::Body)?;
        let lang = fb.get_text(doc, PostField::Lang)?;
        let category = fb.get_text(doc, PostField::Category)?;
        let tags = fb.get_text(doc, PostField::Tags)?;

        let created_at = fb.get_date(doc, PostField::CreatedAt)?;
        let updated_at = fb.get_date(doc, PostField::UpdatedAt)?;
        let created_at_format =
            DateTimeFormat::from(fb.get_text(doc, PostField::CreatedAtFormat)?.as_str());
        let updated_at_format =
            DateTimeFormat::from(fb.get_text(doc, PostField::UpdatedAtFormat)?.as_str());
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

        Ok(Self {
            slug,
            body,
            raw_text: None,
            matter: FrontMatter::new(
                uuid,
                title,
                description,
                category,
                Lang::from_str(&lang).unwrap(),
                tags,
                Some(DateTimeWithFormat::new(created_at, created_at_format)),
                Some(DateTimeWithFormat::new(updated_at, updated_at_format)),
            ),
        })
    }

    pub fn to_doc(
        &self,
        schema: &Schema,
        created_at: &DateTimeWithFormat,
        updated_at: &DateTimeWithFormat,
    ) -> Document {
        let fb = FieldGetter::new(schema);
        let mut doc = Document::new();

        [
            (PostField::Uuid, self.uuid()),
            (PostField::Slug, self.slug()),
            (PostField::Title, self.title()),
            (PostField::Description, self.matter.description()),
            (PostField::Body, self.body()),
            (PostField::Lang, self.lang().as_str().to_string()),
            (PostField::Category, self.matter.category()),
            (PostField::CreatedAtFormat, created_at.format().to_string()),
            (PostField::UpdatedAtFormat, updated_at.format().to_string()),
        ]
        .into_iter()
        .for_each(|(pf, text)| doc.add_text(fb.get_field(pf), text));

        if let Some(raw_text) = self.raw_text() {
            doc.add_text(fb.get_field(PostField::RawText), raw_text);
        }

        let tags = fb.get_field(PostField::Tags);

        let tag_text = match self.matter.tags() {
            Some(tags) => tags.join(" "),
            None => "".to_string(),
        };

        doc.add_text(tags, tag_text);

        doc.add_date(fb.get_field(PostField::CreatedAt), &created_at.datetime());
        doc.add_date(fb.get_field(PostField::UpdatedAt), &updated_at.datetime());

        doc
    }
}

// This is nightly version of rust

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
