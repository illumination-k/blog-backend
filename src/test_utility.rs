extern crate rand;
use std::path::Path;

use anyhow::Result;
use tantivy::Index;
use crate::{posts::{frontmatter::FrontMatter, Lang, Post}, datetime::DateTimeWithFormat, text_engine::{schema::build_schema, index::read_or_build_index, query::put}};
use rand::prelude::IteratorRandom;
use strum::IntoEnumIterator;

const TITLE_LENGTH: usize = 10;
const DESCRIPTION_LENGTH: usize = 100;
const BODY_LENGHT: usize = 1000;
const TAG_CATEGORIES_LENGTH: usize = 10;

#[cfg(not(tarpaulin_include))]
fn rand_string(base: &str, size: usize) -> String {
    let mut rng = &mut rand::thread_rng();
    let mut result: Vec<char> = Vec::new();
    for _ in 0..size {
        result.push(base.chars().choose(&mut rng).unwrap());
    }
    result.into_iter().collect()
}

#[cfg(not(tarpaulin_include))]
fn rand_alpahbet(size: usize) -> String {
    let base = "abcdefghijklmnopqrstuvwstuxyz";

    rand_string(base, size)
}

#[cfg(not(tarpaulin_include))]
fn rand_japanase(size: usize) -> String {
    let base = "いろはにほへとちりぬるをわがよたれぞつねならむabcdefghijk!?*()&^%=+-/";

    rand_string(base, size)
}

#[cfg(not(tarpaulin_include))]
fn rand_lang() -> Lang {
    let mut rng = &mut rand::thread_rng();
    Lang::iter().choose(&mut rng).unwrap()
}

#[cfg(not(tarpaulin_include))]
fn rand_tags(size: usize) -> Option<Vec<String>> {    
    let flag = rand::random::<bool>();

    if flag {
        None
    } else {
        let mut tags = Vec::with_capacity(size);
        for _ in 0..size {
            tags.push(rand_japanase(TAG_CATEGORIES_LENGTH))
        }
        Some(tags)
    }
}

#[cfg(not(tarpaulin_include))]
pub fn rand_matter() -> FrontMatter {
    let tags = rand_tags(3);
    FrontMatter::new(
        uuid::Uuid::new_v4(),
        rand_japanase(TITLE_LENGTH),
        rand_japanase(DESCRIPTION_LENGTH),
        rand_japanase(TAG_CATEGORIES_LENGTH),
        rand_lang(),
        tags,
        Some(DateTimeWithFormat::default()),
        Some(DateTimeWithFormat::default()),
    )
}

#[cfg(not(tarpaulin_include))]
pub fn rand_post() -> Post {
    Post::new(rand_alpahbet(10), rand_matter(), rand_japanase(BODY_LENGHT))
}

#[cfg(not(tarpaulin_include))]
pub fn build_random_posts_index(post_size: usize, index_path: &Path) -> Result<(Vec<Post>, Index)> {
    let posts: Vec<Post> = (0..post_size).map(|_| rand_post()).collect();
    let schema = build_schema();
    let index = read_or_build_index(schema,  index_path, true)?;
    let mut index_writer = index.writer(100_000_000)?;
    for post in posts.iter() {
        put(post, &index, &mut index_writer)?;
    }
    
    Ok((posts, index))
}

