use anyhow::{anyhow, Result};
use itertools::Itertools;
use yaml_rust::{Yaml, YamlLoader};

use crate::utils::Lang;

#[derive(Debug, Clone, PartialEq)]
pub struct FrontMatter {
    id: String,
    title: String,
    description: String,
    lang: Lang,
    category: String,
    tags: Option<Vec<String>>,
}

impl FrontMatter {
    pub fn id(&self) -> String {
        self.id.clone()
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

    pub fn lang(&self) -> &str {
        self.lang.to_str()
    }

    pub fn category(&self) -> String {
        self.category.clone()
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
    let id = doc["id"].as_str().expect("Need Id").to_string();
    let title = doc["title"].as_str().expect("Need Title").to_string();
    let category = doc["category"].as_str().expect("Need Category").to_string();
    let description = doc["description"]
        .as_str()
        .expect("Need Description")
        .to_string();

    let tags = match doc["tags"].as_vec() {
        Some(t) => Some(
            t.into_iter()
                .map(|ss| match ss {
                    Yaml::Integer(i) => i.to_string(),
                    Yaml::String(s) => s.to_owned(),
                    _ => panic!("Unsupported tag type. Tags must be intger or string"),
                })
                .collect_vec(),
        ),
        None => None,
    };

    let lang = match &doc["lang"] {
        Yaml::BadValue => Lang::Ja,
        Yaml::String(s) => Lang::from_str(&s)?,
        _ => panic!("Unsupported lang type. Lang must be string"),
    };

    Ok(FrontMatter {
        id,
        title,
        description,
        category,
        lang,
        tags,
    })
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
    use super::*;
    #[test]
    fn test_detect_frontmatter() {
        let test_string = "---\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\n---\nsomething that's not yaml";
        assert_eq!(find_frontmatter_block(test_string), Some((0, 67)));
    }

    #[test]
    fn test_frontmatter() {
        let test_string = "---\nid: uuid\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\n---\nsomething that's not yaml";

        let (frontmatter, content) = split_frontmatter_and_content(test_string);
        let expect_frontmatter = FrontMatter {
            id: "uuid".to_string(),
            title: "Valid Yaml Test".to_string(),
            description: "Valid Yaml Description".to_string(),
            category: "Valid Yaml category".to_string(),
            lang: Lang::Ja,
            tags: None,
        };
        assert_eq!(frontmatter.unwrap(), expect_frontmatter);
        assert_eq!(content, "something that's not yaml")
    }

    #[test]
    fn test_frontmatter_tags() {
        let test_string_tags = "---id: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- '1'\n- '2'\n---\nsomething that's not yaml";
        let test_int_tags = "---id: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- 1\n- 2\n---\nsomething that's not yaml";
        let (string_frontmatter, _) = split_frontmatter_and_content(test_string_tags);
        let (int_frontmatter, _) = split_frontmatter_and_content(test_int_tags);
        assert_eq!(string_frontmatter, int_frontmatter);
    }
}
