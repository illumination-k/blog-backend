use uuid::Uuid;

use super::{dump::dump_matter, frontmatter::FrontMatter};
use crate::posts::Lang;
use anyhow::Result;

pub fn template() -> Result<String> {
    let matter = FrontMatter::new(
        Uuid::new_v4().to_string().as_str(),
        "",
        "",
        "",
        Lang::Ja,
        None,
        None,
        None,
    );

    dump_matter(&matter)
}
