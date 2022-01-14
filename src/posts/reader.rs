use crate::posts::Post;
use anyhow::Result;
use glob::glob;
use itertools::Itertools;
use std::path::PathBuf;

pub fn get_all_posts(glob_pattern: &str) -> Result<Vec<(PathBuf, Post)>> {
    let posts = glob(glob_pattern)?
        .into_iter()
        .filter_map(|path| path.ok())
        .filter_map(|path| {
            // should be /path/to/filename.md
            let post = Post::from_path(&path);
            if let Ok(post) = post {
                Some((path, post))
            } else {
                error!("Error in {:?}. Maybe this file has invalid frontmatter. Skipping this file.", path);
                None
            }
        })
        .collect_vec();

    Ok(posts)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::markdown::template::template;
    use std::fs;
    use std::io::Write;
    #[test]
    fn test_get_all_posts() {
        let temp_dir = tempdir::TempDir::new("test_get_all_posts").unwrap();

        let dirs = &["a", "b", "b/d", "c", "c/f", "c/f/h"];
        let md_files = &[
            vec!["1.md", "2.md"],
            vec!["3.md", "not_md.txt"],
            vec!["4.md", "5.md", "6.md"],
            vec!["7.md", "testing.png"],
            vec!["9.md", "10.md"],
            vec!["11.md"],
        ];
        for (dir, files) in dirs.into_iter().zip(md_files.into_iter()) {
            let dir_path = temp_dir.path().join(dir);
            fs::create_dir(&dir_path).unwrap();
            for file in files.iter() {
                let mut f = fs::File::create(&dir_path.join(file)).unwrap();
                write!(f, "{}", template(&false, &None).unwrap()).unwrap();
            }
        }

        let expect_files_count = md_files
            .iter()
            .flatten()
            .filter(|x| x.ends_with(".md"))
            .count();
        let actual_files_count = get_all_posts(&format!("{}/**/*.md", temp_dir.path().display()))
            .unwrap()
            .len();

        assert_eq!(expect_files_count, actual_files_count);
    }
}
