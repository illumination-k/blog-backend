use std::fs;
use std::path::PathBuf;

use lindera::tokenizer::TokenizerConfig;
use lindera_core::viterbi::{Mode, Penalty};
use lindera_tantivy::tokenizer::LinderaTokenizer;
use tantivy::schema::*;
use tantivy::Index;

use tantivy::Result;

use crate::utils::Lang;

pub fn build_index(schema: Schema, index_dir: &PathBuf) -> Result<Index> {
    let tokenizer_name = Lang::Ja.tokenizer_name();

    let index = Index::create_in_dir(index_dir, schema)?;

    let config = TokenizerConfig {
        dict_path: None,
        user_dict_path: None,
        user_dict_bin_path: None,
        mode: Mode::Decompose(Penalty::default()),
    };

    // register Lindera tokenizer
    index.tokenizers().register(
        &tokenizer_name,
        LinderaTokenizer::with_config(config).unwrap(),
    );

    Ok(index)
}

pub fn read_or_build_index(schema: Schema, index_dir: &PathBuf, rebuild: bool) -> Result<Index> {
    if index_dir.exists() {
        if rebuild {
            fs::remove_dir_all(index_dir)?;
            fs::create_dir(index_dir)?;
            build_index(schema, index_dir)
        } else {
            Index::open_in_dir(index_dir)
        }
    } else {
        fs::create_dir(index_dir)?;
        build_index(schema, index_dir)
    }
}
