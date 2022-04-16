use anyhow::{anyhow, Result};
use linked_hash_map::LinkedHashMap;
use yaml_rust::Yaml;

use crate::{datetime::DateTimeWithFormat, text_engine::schema::PostField};

use super::FrontMatter;

pub(super) fn get_str_from_yaml(doc: &Yaml, field: PostField) -> Result<String> {
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

pub(super) fn parse_date_from_yaml(
    doc: &Yaml,
    key: PostField,
) -> Result<Option<DateTimeWithFormat>> {
    doc[key.as_str()]
        .as_str()
        .map_or(Ok(None), |s| match DateTimeWithFormat::from_str(s) {
            Ok(df) => Ok(Some(df)),
            Err(e) => Err(e),
        })
}

pub(super) fn get_tags_from_yaml(doc: &Yaml) -> Option<Vec<String>> {
    doc[PostField::Tags.as_str()].as_vec().map(|t| {
        t.iter()
            .map(|ss| match ss {
                Yaml::Integer(i) => i.to_string(),
                Yaml::String(s) => s.to_owned(),
                _ => panic!("Unsupported tag type. Tags must be intger or string"),
            })
            .collect()
    })
}

pub(super) fn get_or_fill_str_from_yaml<S: ToString>(
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

pub(super) fn matter_to_yaml(matter: &FrontMatter) -> Yaml {
    fn insert_to_yamlmap<S: ToString>(k: S, v: String, lm: &mut LinkedHashMap<Yaml, Yaml>) {
        lm.insert(Yaml::String(k.to_string()), Yaml::String(v));
    }

    // To preserve order, need to use linked-hash-map
    let map: LinkedHashMap<&str, String> = [
        (PostField::Uuid.as_str(), matter.uuid()),
        (PostField::Title.as_str(), matter.title()),
        (PostField::Description.as_str(), matter.description()),
        (PostField::Lang.as_str(), matter.lang().as_str().to_string()),
        (PostField::Category.as_str(), matter.category()),
    ]
    .into_iter()
    .collect();

    let opmap: LinkedHashMap<&str, Option<String>> = [
        (
            PostField::CreatedAt.as_str(),
            matter.created_at().map(|c| c.to_string()),
        ),
        (
            PostField::UpdatedAt.as_str(),
            matter.updated_at().map(|u| u.to_string()),
        ),
    ]
    .into_iter()
    .collect();

    let mut lm = LinkedHashMap::new();

    // Preserve insert order
    for (k, v) in map.into_iter() {
        insert_to_yamlmap(k, v, &mut lm);
    }

    if let Some(tags) = matter.tags() {
        let tags = tags.into_iter().map(Yaml::String).collect();
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

#[cfg(test)]
mod test {
    use crate::test_utility::rand_matter;

    use super::*;

    #[test]
    fn test_basic() -> Result<()> {
        let matter = rand_matter();

        let yaml = matter_to_yaml(&matter);

        [
            PostField::Uuid,
            PostField::Title,
            PostField::Category,
            PostField::Description,
        ]
        .into_iter()
        .for_each(|pf| assert_eq!(get_str_from_yaml(&yaml, pf).unwrap(), matter.get_text(pf)));

        Ok(())
    }
}
