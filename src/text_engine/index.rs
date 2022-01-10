use std::fs;
use std::path::Path;

use lindera::tokenizer::TokenizerConfig;
use lindera_core::viterbi::{Mode, Penalty};
use lindera_tantivy::tokenizer::LinderaTokenizer;
use tantivy::schema::*;
use tantivy::Index;

use tantivy::tokenizer::LowerCaser;
use tantivy::tokenizer::RawTokenizer;
use tantivy::tokenizer::TextAnalyzer;
use tantivy::Result;

use crate::posts::Lang;

pub fn read_or_build_index(schema: Schema, index_dir: &Path, rebuild: bool) -> Result<Index> {
    let index = if index_dir.exists() {
        if rebuild {
            fs::remove_dir_all(index_dir)?;
            fs::create_dir(index_dir)?;
            Index::create_in_dir(index_dir, schema)
        } else {
            Index::open_in_dir(index_dir)
        }
    } else {
        fs::create_dir(index_dir)?;
        Index::create_in_dir(index_dir, schema)
    }?;

    let config = TokenizerConfig {
        dict_path: None,
        user_dict_path: None,
        user_dict_bin_path: None,
        mode: Mode::Decompose(Penalty::default()),
    };

    index.tokenizers().register("raw_tokenizer", RawTokenizer);
    let tokenizer_name = Lang::Ja.tokenizer_name();
    let ja_tokenizer =
        TextAnalyzer::from(LinderaTokenizer::with_config(config).unwrap()).filter(LowerCaser);
    // register Lindera tokenizer
    index.tokenizers().register(&tokenizer_name, ja_tokenizer);

    Ok(index)
}
