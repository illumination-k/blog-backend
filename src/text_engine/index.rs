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
            let index = Index::open_in_dir(index_dir);

            // if index is not exist in index_dir, we should create new index
            if index.is_ok() {
                index
            } else {
                Index::create_in_dir(index_dir, schema)
            }
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

#[cfg(test)]
mod test_index {
    use super::read_or_build_index;
    use crate::build_schema;
    use tempdir::TempDir;

    #[test]
    fn test_read_or_buld_index_with_schema() {
        let temp_dir = TempDir::new("test_read_or_buld_index_with_schema").unwrap();

        let schema = build_schema();
        let build = read_or_build_index(schema.clone(), temp_dir.path(), false);
        let read = read_or_build_index(schema.clone(), temp_dir.path(), false);
        let rebuild = read_or_build_index(schema, temp_dir.path(), true);

        assert!(build.is_ok());
        assert!(read.is_ok());
        assert!(rebuild.is_ok());
    }
}
