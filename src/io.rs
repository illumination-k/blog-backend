use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
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

pub fn write_string<P: AsRef<Path>>(p: &P, string: &str) -> Result<()> {
    let file = File::create(&p).expect("File create error!");
    let mut w = BufWriter::new(&file);
    write!(w, "{}", string)?;
    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod test_io {
    use super::*;
    #[test]
    fn not_found() {
        assert!(read_string("not_found.md").is_err());
    }

    #[test]
    fn test_write_and_read() {
        let temp_dir = tempdir::TempDir::new("io_test_write_and_read").unwrap();
        let temp_path = temp_dir.path().join("a.md");

        let write = "aaa\nbbb\n";
        let res = write_string(&temp_path, write);
        assert!(res.is_ok());
        let read = read_string(&temp_path);
        assert!(read.is_ok());
        assert_eq!(write, read.unwrap());
    }
}
