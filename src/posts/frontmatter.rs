use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use uuid::Uuid;
use yaml_rust::{Yaml, YamlLoader};

use crate::{
    posts::Lang,
    text_engine::{
        datetime::{parse_datetime, DateTimeFormat},
        schema::PostField,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct DateTimeWithFormat {
    datetime: DateTime<Utc>,
    format: DateTimeFormat,
}

impl DateTimeWithFormat {
    pub fn new(datetime: DateTime<Utc>, format: DateTimeFormat) -> Self {
        Self { datetime, format }
    }

    pub fn now(format: &DateTimeFormat) -> Self {
        Self::new(Utc::now(), format.to_owned())
    }

    pub fn datetime(&self) -> DateTime<Utc> {
        self.datetime
    }

    pub fn format(&self) -> DateTimeFormat {
        self.format.clone()
    }
}

impl ToString for DateTimeWithFormat {
    fn to_string(&self) -> String {
        self.format.format(self.datetime)
    }
}

impl Default for DateTimeWithFormat {
    fn default() -> Self {
        Self::now(&DateTimeFormat::RFC3339)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FrontMatter {
    uuid: String,
    title: String,
    description: String,
    lang: Lang,
    category: String,
    tags: Option<Vec<String>>,
    created_at: Option<DateTimeWithFormat>,
    updated_at: Option<DateTimeWithFormat>,
}

impl FrontMatter {
    #[allow(clippy::too_many_arguments)]
    pub fn new<S1, S2, S3, S4>(
        uuid: S1,
        title: S2,
        description: S3,
        category: S4,
        lang: Lang,
        tags: Option<Vec<String>>,
        created_at: Option<DateTimeWithFormat>,
        updated_at: Option<DateTimeWithFormat>,
    ) -> Self
    where
        S1: ToString,
        S2: ToString,
        S3: ToString,
        S4: ToString,
    {
        Self {
            uuid: uuid.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            lang,
            tags,
            created_at,
            updated_at,
        }
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }
    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn tags(&self) -> Option<Vec<String>> {
        self.tags.clone()
    }

    pub fn lang(&self) -> Lang {
        self.lang.clone()
    }

    pub fn category(&self) -> String {
        self.category.clone()
    }

    pub fn created_at(&self) -> Option<DateTimeWithFormat> {
        self.created_at.to_owned()
    }

    pub fn updated_at(&self) -> Option<DateTimeWithFormat> {
        self.updated_at.to_owned()
    }

    /// **CAUSION!**  
    /// This function do not return strict equal.
    /// If updated_at and created_at in `self.matter` is `None`,
    /// this function do not compare updated_at and created_at.  
    /// It is useful when comparing the post from doc and
    /// the post which has no updated_at and created_at field.  
    pub fn equal_matter_from_doc(&self, other: &FrontMatter) -> bool {
        let mut flag = self.uuid == other.uuid
            && self.title == other.title
            && self.description == other.description
            && self.category == other.category
            && self.tags == other.tags
            && self.lang == other.lang;

        if self.created_at.is_some() {
            flag = flag && self.created_at == other.created_at
        }

        // do not compare updated at!

        flag
    }

    pub fn to_yaml(&self) -> Yaml {
        fn insert_to_yamlmap<S: ToString>(k: S, v: String, lm: &mut LinkedHashMap<Yaml, Yaml>) {
            lm.insert(Yaml::String(k.to_string()), Yaml::String(v));
        }

        // To preserve order, we should use linked-hash-map
        let map: LinkedHashMap<&str, String> = [
            (PostField::Uuid.as_str(), self.uuid()),
            (PostField::Title.as_str(), self.title()),
            (PostField::Description.as_str(), self.description()),
            (PostField::Lang.as_str(), self.lang.as_str().to_string()),
            (PostField::Category.as_str(), self.category()),
        ]
        .into_iter()
        .collect();

        let opmap: LinkedHashMap<&str, Option<String>> = [
            (
                PostField::CreatedAt.as_str(),
                self.created_at.to_owned().map(|c| c.to_string()),
            ),
            (
                PostField::UpdatedAt.as_str(),
                self.updated_at.to_owned().map(|u| u.to_string()),
            ),
        ]
        .into_iter()
        .collect();

        let mut lm = LinkedHashMap::new();

        // Preserve insert order
        for (k, v) in map.into_iter() {
            insert_to_yamlmap(k, v, &mut lm);
        }

        if let Some(tags) = self.tags() {
            let tags = tags.into_iter().map(Yaml::String).collect_vec();
            lm.insert(
                Yaml::String(PostField::Tags.as_str().to_string()),
                Yaml::Array(tags),
            );
        }

        for (k, v) in opmap.into_iter() {
            if let Some(v) = v {
                insert_to_yamlmap(k, v, &mut lm);
            }
        }

        Yaml::Hash(lm)
    }
}

pub fn find_frontmatter_block(text: &str) -> Option<(usize, usize)> {
    match text.starts_with("---\n") {
        true => {
            let slice_after_marker = &text[4..];
            let fm_end = match slice_after_marker.find("---\n") {
                Some(f) => f,
                None => return None,
            };

            Some((0, fm_end + 2 * 4))
        }
        false => None,
    }
}

fn get_str_from_yaml(doc: &Yaml, field: PostField) -> Result<String> {
    let field_str = field.as_str();
    match &doc[field_str] {
        Yaml::String(s) => Ok(s.to_owned()),
        Yaml::Integer(i) => Ok(i.to_string()),
        _ => Err(anyhow!(format!(
            "{} field is need in frontmatter",
            field_str
        ))),
    }
}

pub fn parse_date_with_format(date_string: &str) -> DateTimeWithFormat {
    match parse_datetime(date_string, None) {
        Ok((format, datetime)) => DateTimeWithFormat { datetime, format },
        Err(e) => panic!("{}", e),
    }
}

pub fn parse_date_from_yaml(doc: &Yaml, key: &str) -> Option<DateTimeWithFormat> {
    doc[key].as_str().map(|s| match parse_datetime(s, None) {
        Ok((format, datetime)) => DateTimeWithFormat { datetime, format },
        Err(e) => panic!("{}", e),
    })
}

pub fn get_or_fill_str_from_yaml<S: ToString>(
    doc: &Yaml,
    field: PostField,
    val: &Option<String>,
    fill_val: S,
) -> String {
    if let Some(val) = val {
        val.to_owned()
    } else if let Ok(val) = get_str_from_yaml(doc, field) {
        val
    } else {
        fill_val.to_string()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn replace_frontmatter(
    frontmatter: &str,
    uuid: &Option<String>,
    title: &Option<String>,
    category: &Option<String>,
    lang: &Option<String>,
    description: &Option<String>,
    tags: &Option<Vec<String>>,
    created_at: &Option<DateTimeWithFormat>,
    updated_at: &Option<DateTimeWithFormat>,
) -> Result<FrontMatter> {
    let docs = if frontmatter.is_empty() {
        YamlLoader::load_from_str("empty: f")?
    } else {
        YamlLoader::load_from_str(frontmatter)?
    };
    let doc = &docs[0];
    let uuid = get_or_fill_str_from_yaml(doc, PostField::Uuid, uuid, Uuid::new_v4());
    let title = get_or_fill_str_from_yaml(doc, PostField::Title, title, "");
    let category = get_or_fill_str_from_yaml(doc, PostField::Category, category, "");
    let description = get_or_fill_str_from_yaml(doc, PostField::Description, description, "");
    let lang = Lang::from_str(&get_or_fill_str_from_yaml(doc, PostField::Lang, lang, "ja"))?;

    let tags = if let Some(tags) = tags {
        Some(tags.to_owned())
    } else {
        doc["tags"].as_vec().map(|t| {
            t.iter()
                .map(|ss| match ss {
                    Yaml::Integer(i) => i.to_string(),
                    Yaml::String(s) => s.to_owned(),
                    _ => panic!("Unsupported tag type. Tags must be intger or string"),
                })
                .collect_vec()
        })
    };

    let created_at = if let Some(created_at) = created_at {
        Some(created_at.to_owned())
    } else {
        parse_date_from_yaml(doc, "created_at")
    };

    let updated_at = if let Some(updated_at) = updated_at {
        Some(updated_at.to_owned())
    } else {
        parse_date_from_yaml(doc, "updated_at")
    };

    Ok(FrontMatter::new(
        uuid,
        title,
        description,
        category,
        lang,
        tags,
        created_at,
        updated_at,
    ))
}

pub fn parse_frontmatter(frontmatter: &str) -> Result<FrontMatter> {
    let docs = YamlLoader::load_from_str(frontmatter)?;
    let doc = &docs[0];
    let uuid = get_str_from_yaml(doc, PostField::Uuid)?;
    let title = get_str_from_yaml(doc, PostField::Title)?;
    let category = get_str_from_yaml(doc, PostField::Category)?;
    let description = get_str_from_yaml(doc, PostField::Description)?;

    let tags = doc["tags"].as_vec().map(|t| {
        t.iter()
            .map(|ss| match ss {
                Yaml::Integer(i) => i.to_string(),
                Yaml::String(s) => s.to_owned(),
                _ => panic!("Unsupported tag type. Tags must be intger or string"),
            })
            .collect_vec()
    });

    let lang = match &doc["lang"] {
        Yaml::BadValue => Lang::Ja,
        Yaml::String(s) => Lang::from_str(s)?,
        _ => panic!("Unsupported lang type. Lang must be string"),
    };

    let created_at = parse_date_from_yaml(doc, "created_at");
    let updated_at = parse_date_from_yaml(doc, "updated_at");

    Ok(FrontMatter::new(
        uuid,
        title,
        description,
        category,
        lang,
        tags,
        created_at,
        updated_at,
    ))
}

pub fn split_frontmatter_and_content(text: &str) -> (Option<FrontMatter>, &str) {
    match find_frontmatter_block(text) {
        Some((fm_start, fm_end)) => (
            Some(parse_frontmatter(&text[fm_start..fm_end]).unwrap()),
            &text[fm_end..],
        ),
        None => (None, text),
    }
}

#[cfg(test)]
mod test {
    use yaml_rust::YamlEmitter;

    use super::*;
    #[test]
    fn test_detect_frontmatter() {
        let test_string = "---\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\n---\nsomething that's not yaml";
        assert_eq!(find_frontmatter_block(test_string), Some((0, 67)));
    }

    #[test]
    fn test_frontmatter() {
        let test_string = "---\nuuid: uuid\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\n---\nsomething that's not yaml";

        let (frontmatter, content) = split_frontmatter_and_content(test_string);
        let expect_frontmatter = FrontMatter::new(
            "uuid".to_string(),
            "Valid Yaml Test".to_string(),
            "Valid Yaml Description".to_string(),
            "Valid Yaml category".to_string(),
            Lang::Ja,
            None,
            None,
            None,
        );
        assert_eq!(frontmatter.unwrap(), expect_frontmatter);
        assert_eq!(content, "something that's not yaml")
    }

    #[test]
    fn test_frontmatter_tags() {
        let test_string_tags = "---\nuuid: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- '1'\n- '2'\n---\nsomething that's not yaml";
        let test_int_tags = "---\nuuid: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- 1\n- 2\n---\nsomething that's not yaml";
        let (string_frontmatter, _) = split_frontmatter_and_content(test_string_tags);
        let (int_frontmatter, _) = split_frontmatter_and_content(test_int_tags);
        assert_eq!(
            string_frontmatter.expect("error in string"),
            int_frontmatter.expect("error in int")
        );
    }

    #[test]
    fn test_frontmatter_to_yaml() {
        let test_string_tags = "---
category: Test
uuid: fabe88b5-a35e-4954-bfd8-b5e88c585e7a
title: Test Markdown
description: Test
lang: ja
created_at: \"2022-01-09T18:10:39+00:00\"
updated_at: \"2022-01-09T18:10:39+00:00\"
tags:
  - \"1\"
  - \"2\"
---

## TEST
";
        let (frontmatter, _) = split_frontmatter_and_content(test_string_tags);
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter
            .dump(&frontmatter.clone().unwrap().to_yaml())
            .unwrap();
        out_str.push_str("\n---\n");
        let (out_frontmatter, _) = split_frontmatter_and_content(&out_str);
        assert_eq!(frontmatter.unwrap(), out_frontmatter.unwrap());
    }

    #[test]
    fn test_equal_from_post() {
        let test_with_date = "---
category: Test
uuid: fabe88b5-a35e-4954-bfd8-b5e88c585e7a
title: Test Markdown
description: Test
lang: ja
created_at: \"2022-01-09T18:10:39+00:00\"
updated_at: \"2022-01-09T18:10:39+00:00\"
tags:
    - \"1\"
    - \"2\"
---
";

        let test_no_date = "---
category: Test
uuid: fabe88b5-a35e-4954-bfd8-b5e88c585e7a
title: Test Markdown
description: Test
lang: ja
tags:
    - \"1\"
    - \"2\"
---
";
        let (frontmatter_with_date, _) = split_frontmatter_and_content(test_with_date);
        let (frontmatter_no_date, _) = split_frontmatter_and_content(test_no_date);
        let frontmatter_with_date = frontmatter_with_date.unwrap();
        let frontmatter_no_date = frontmatter_no_date.unwrap();
        assert!(frontmatter_no_date.equal_matter_from_doc(&frontmatter_no_date));
        assert!(frontmatter_with_date.equal_matter_from_doc(&frontmatter_with_date));
        assert!(frontmatter_no_date.equal_matter_from_doc(&frontmatter_with_date));

        enum TestCase {
            Title,
            UpdatedAt,
        }
        let tests = [TestCase::Title, TestCase::UpdatedAt];

        for test in tests.iter() {
            match test {
                TestCase::Title => {
                    let mut updated_frontmatter_with_date = frontmatter_with_date.clone();
                    updated_frontmatter_with_date.title = "New Test Title".into();
                    assert!(!frontmatter_with_date
                        .equal_matter_from_doc(&updated_frontmatter_with_date));
                }
                TestCase::UpdatedAt => {
                    let mut updated_frontmatter_with_date = frontmatter_with_date.clone();
                    updated_frontmatter_with_date.updated_at =
                        Some(DateTimeWithFormat::new(Utc::now(), DateTimeFormat::RFC3339));
                    assert!(
                        frontmatter_with_date.equal_matter_from_doc(&updated_frontmatter_with_date)
                    );
                }
            }
        }
    }
}
