use crate::{structure::class::{ ClassFile, ClassFileParser }, util::{
  hex::{ hex_viewer },
}};

use std::fs::File;
use std::io::{self, Read, Result};
use nom::{error::Error, Err};
use nom::error::ErrorKind;
use nom::{
  IResult,
};

fn parse_all(input: &[u8]) -> IResult<&[u8], ClassFile> {
  let mut parser = ClassFileParser::new();
  let class_file = parser.parse(input);
  match class_file {
    Ok((remaining, class_file)) => {
      if !remaining.is_empty() {
        Err(Err::Error(Error::new(remaining, ErrorKind::Eof)))
      } else {
        Ok((remaining, class_file))
      }
    },
    Err(e) => Err(e),
  }
}

pub fn read_file(path: &str) -> Result<ClassFile> {
  let mut file = File::open(&path)?;
  let metadata = file.metadata()?;
  println!("File size: {} bytes", metadata.len());
  let mut bytes = vec![];
  let _ = file.read_to_end(&mut bytes);
  let hex_string: String = hex_viewer(&bytes);
  println!("hex: \n{}", hex_string);

  match parse_all(&bytes) {
    Ok((_, class_file)) => Ok(class_file),
    Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse class file")),
  }
}