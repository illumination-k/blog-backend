use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use maplit::hashmap;
use yaml_rust::{Yaml, YamlLoader};

use crate::posts::Lang;

#[derive(Debug, Clone, PartialEq)]
pub struct FrontMatter {
    uuid: String,
    title: String,
    description: String,
    lang: Lang,
    category: String,
    tags: Option<Vec<String>>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl FrontMatter {
    #[allow(clippy::too_many_arguments)]
    pub fn new<S: ToString>(
        uuid: S,
        title: S,
        description: S,
        category: S,
        lang: Lang,
        tags: Option<Vec<String>>,
        created_at: Option<DateTime<Utc>>,
        updated_at: Option<DateTime<Utc>>,
    ) -> Self {
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

    pub fn created_at(&self) -> Option<DateTime<Utc>> {
        self.created_at
    }

    pub fn updated_at(&self) -> Option<DateTime<Utc>> {
        self.updated_at
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

        if self.updated_at.is_some() {
            flag = flag && self.updated_at == other.updated_at
        }

        flag
    }

    pub fn to_yaml(&self) -> Yaml {
        fn insert_to_yamlmap<S: ToString>(k: S, v: String, lm: &mut LinkedHashMap<Yaml, Yaml>) {
            lm.insert(Yaml::String(k.to_string()), Yaml::String(v));
        }

        let map = hashmap! {
            "uuid" => self.uuid(),
            "title" => self.title(),
            "description" => self.description(),
            "lang" => self.lang().as_str().to_string(),
            "category" => self.category(),
        };

        let opmap = hashmap! {
            "created_at" => self.created_at,
            "updated_at" => self.updated_at,
        };

        let mut lm = LinkedHashMap::new();

        for (k, v) in map.into_iter() {
            insert_to_yamlmap(k, v, &mut lm);
        }

        for (k, v) in opmap.into_iter() {
            if let Some(date) = v {
                insert_to_yamlmap(k, date.to_rfc3339(), &mut lm);
            }
        }

        if let Some(tags) = self.tags() {
            let tags = tags.into_iter().map(Yaml::String).collect_vec();
            lm.insert(Yaml::String("tags".to_string()), Yaml::Array(tags));
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

pub fn parse_frontmatter(frontmatter: &str) -> Result<FrontMatter> {
    let docs = YamlLoader::load_from_str(frontmatter)?;
    let doc = &docs[0];
    let uuid = doc["uuid"].as_str().expect("Need Id").to_string();
    let title = doc["title"].as_str().expect("Need Title").to_string();
    let category = doc["category"].as_str().expect("Need Category").to_string();
    let description = doc["description"]
        .as_str()
        .expect("Need Description")
        .to_string();

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

    let created_at = doc["created_at"].as_str().map(|s| {
        match DateTime::parse_from_rfc3339(s)
            .with_context(|| "created at should be rfc3339")
        {
            Ok(date) => date.into(),
            Err(e) => panic!("{}", e),
        }
    });

    let updated_at = doc["updated_at"].as_str().map(|s| {
        match DateTime::parse_from_rfc3339(s)
            .with_context(|| "updated at should be rfc3339")
        {
            Ok(date) => date.into(),
            Err(e) => panic!("{}", e),
        }
    });

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
                    updated_frontmatter_with_date.updated_at = Some(Utc::now());
                    assert!(!frontmatter_with_date
                        .equal_matter_from_doc(&updated_frontmatter_with_date));
                }
            }
        }
    }
}
