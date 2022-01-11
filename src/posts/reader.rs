use crate::posts::Post;
use anyhow::Result;
use glob::glob;
use itertools::Itertools;
use std::path::PathBuf;

pub fn get_all_posts(glob_pattern: &str) -> Result<Vec<(PathBuf, Post)>> {
    let posts = glob(glob_pattern)?
        .into_iter()
        .filter_map(|path| path.ok())
        .map(|path| {
            // should be /path/to/filename.md
            let post = Post::from_path(&path);
            (path, post)
        })
        .collect_vec();

    Ok(posts)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_all_posts() {
        dbg!(&get_all_posts("./test/posts/**/*.md").unwrap());
    }
}
