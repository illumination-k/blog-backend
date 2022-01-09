use std::{
    fs::File,
    io::{BufReader, Read, BufWriter, Write},
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