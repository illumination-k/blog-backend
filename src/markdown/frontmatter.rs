use anyhow::Result;
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
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
}

impl FrontMatter {
    pub fn new<S: ToString>(
        uuid: S,
        title: S,
        description: S,
        category: S,
        lang: Lang,
        tags: Option<Vec<String>>,
    ) -> Self {
        Self {
            uuid: uuid.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            lang,
            tags,
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

    pub fn to_yaml_with_date(&self, created_at: String, updated_at: String) -> Yaml {
        let mut lm = LinkedHashMap::new();
        lm.insert(Yaml::String("uuid".to_string()), Yaml::String(self.uuid()));
        lm.insert(
            Yaml::String("title".to_string()),
            Yaml::String(self.title()),
        );
        lm.insert(
            Yaml::String("description".to_string()),
            Yaml::String(self.description()),
        );
        lm.insert(
            Yaml::String("lang".to_string()),
            Yaml::String(self.lang().as_str().to_string()),
        );
        lm.insert(
            Yaml::String("category".to_string()),
            Yaml::String(self.category()),
        );
        lm.insert(
            Yaml::String("created_at".to_string()),
            Yaml::String(created_at),
        );
        lm.insert(
            Yaml::String("updated_at".to_string()),
            Yaml::String(updated_at),
        );
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
    let uuid = doc["id"].as_str().expect("Need Id").to_string();
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

    Ok(FrontMatter {
        uuid,
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
    use chrono::Utc;
    use yaml_rust::YamlEmitter;

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
            uuid: "uuid".to_string(),
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
        let test_string_tags = "---\nid: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- '1'\n- '2'\n---\nsomething that's not yaml";
        let test_int_tags = "---\nid: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- 1\n- 2\n---\nsomething that's not yaml";
        let (string_frontmatter, _) = split_frontmatter_and_content(test_string_tags);
        let (int_frontmatter, _) = split_frontmatter_and_content(test_int_tags);
        assert_eq!(
            string_frontmatter.expect("error in string"),
            int_frontmatter.expect("error in int")
        );
    }

    #[test]
    fn test_frontmatter_to_yaml() {
        let test_string_tags = "---\nid: uuid\n\ntitle: Valid Yaml Test\ndescription: Valid Yaml Description\ncategory: Valid Yaml category\ntags:\n- '1'\n- '2'\n---\nsomething that's not yaml";
        let (string_frontmatter, _) = split_frontmatter_and_content(test_string_tags);
        dbg!(&string_frontmatter);
        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter
            .dump(
                &string_frontmatter
                    .unwrap()
                    .to_yaml_with_date(Utc::now().to_rfc3339(), Utc::now().to_rfc3339()),
            )
            .unwrap();
        out_str.push_str("---\n");
        dbg!(&out_str);
    }
}
