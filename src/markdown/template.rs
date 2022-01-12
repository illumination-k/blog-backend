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

#[cfg(test)]
mod test_template {
    use std::io::Write;
    use tempdir::TempDir;
    use super::template;

    #[test]
    fn test_parse_basic_template() {
        use std::fs;

        use crate::posts::Post;

        let template = template(&false, &None).unwrap();

        let temp_dir = TempDir::new("test_parse_basic_template").unwrap();
        let temp_file = temp_dir.path().join("a.md");
        let mut f = fs::File::create(&temp_file).unwrap();
        write!(f, "{}", template).unwrap();

        let _ = Post::from_path(&temp_file);
    }

    #[test]
    fn test_parse_with_date_template() {
        use std::fs;

        use crate::posts::Post;

        let template = template(&true, &None).unwrap();

        let temp_dir = TempDir::new("test_parse_with_date_template").unwrap();
        let temp_file = temp_dir.path().join("a.md");
        let mut f = fs::File::create(&temp_file).unwrap();
        write!(f, "{}", template).unwrap();

        let _ = Post::from_path(&temp_file);
    }
}
