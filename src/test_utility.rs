extern crate rand;
use std::path::Path;

use crate::{
    datetime::DateTimeWithFormat,
    posts::{frontmatter::FrontMatter, Lang, Post},
    text_engine::{index::read_or_build_index, query::put, schema::build_schema},
};
use anyhow::Result;
use rand::prelude::IteratorRandom;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use tantivy::Index;

pub const TITLE_LENGTH: usize = 10;
pub const DESCRIPTION_LENGTH: usize = 100;
pub const BODY_LENGHT: usize = 1000;
pub const TAG_CATEGORIES_LENGTH: usize = 10;

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
pub fn rand_alpahbet(size: usize) -> String {
    let base = "abcdefghijklmnopqrstuvwstuxyz";

    rand_string(base, size)
}

#[cfg(not(tarpaulin_include))]
pub fn rand_japanase(size: usize) -> String {
    let base = "いろはにほへとちりぬるをわがよたれぞつねならむabcdefghijk!?*()&^%=+-/";

    rand_string(base, size)
}

#[cfg(not(tarpaulin_include))]
pub fn rand_lang() -> Lang {
    let mut rng = &mut rand::thread_rng();
    Lang::iter().choose(&mut rng).unwrap()
}

#[cfg(not(tarpaulin_include))]
pub fn rand_tags(size: usize) -> Option<Vec<String>> {
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
    let index = read_or_build_index(schema, index_path, true)?;
    let mut index_writer = index.writer(100_000_000)?;
    for post in posts.iter() {
        put(post, &index, &mut index_writer)?;
    }

    Ok((posts, index))
}

#[cfg(not(tarpaulin_include))]
#[derive(Debug, Serialize, Deserialize)]
pub struct PostResponse {
    pub uuid: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub lang: String,
    pub tags: Vec<String>,
    pub body: String,
    pub created_at: String,
    pub updated_at: String,
}

impl PartialEq<Post> for PostResponse {
    fn eq(&self, other: &Post) -> bool {
        let mut flag = self.uuid == other.uuid()
            && self.slug == other.slug()
            && self.title == other.title()
            && self.description == other.description()
            && self.category == other.category()
            && self.lang == other.lang().to_string()
            && self.body == other.body();

        // if post tags is None, postresponse is empty vec
        if let Some(tags) = other.tags() {
            flag = flag && tags == self.tags;
        } else {
            flag = flag && self.tags.is_empty()
        }

        if let Some(created_at) = other.created_at() {
            flag = flag && created_at.to_string() == self.created_at;
        }

        if let Some(updated_at) = other.updated_at() {
            flag = flag && updated_at.to_string() == self.updated_at;
        }

        flag
    }
}
