use uuid::Uuid;

use super::{
    dump::dump_matter,
    frontmatter::{DateTimeWithFormat, FrontMatter},
};
use crate::{posts::Lang, text_engine::datetime::DateTimeFormat};
use anyhow::Result;

pub fn template(with_date: &bool, datetime_format: &Option<String>) -> Result<String> {
    let fmt = if let Some(s) = datetime_format {
        DateTimeFormat::from(s.as_str())
    } else {
        DateTimeFormat::RFC3339
    };

    let (created_at, updated_at) = if *with_date {
        (
            Some(DateTimeWithFormat::now(&fmt)),
            Some(DateTimeWithFormat::now(&fmt)),
        )
    } else {
        (None, None)
    };

    let matter = FrontMatter::new(
        Uuid::new_v4().to_string().as_str(),
        "",
        "",
        "",
        Lang::Ja,
        None,
        created_at,
        updated_at,
    );

    dump_matter(&matter)
}
