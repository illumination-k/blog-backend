pub mod extract_text;
pub mod frontmatter;
pub mod template;

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use anyhow::{Context, Result};

pub fn read_string<P: AsRef<Path>>(p: P) -> Result<String> {
    let mut buf = String::new();
    let mut file = File::open(&p)
        .map(BufReader::new)
        .with_context(|| format!("{:?} is not found", p.as_ref()))?;

    file.read_to_string(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn not_found() {
        let err = read_string("not_found.md");
        dbg!(&err);
    }
}
