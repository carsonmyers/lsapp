#![feature(type_alias_impl_trait)]

mod parser;

use std::collections::HashMap;
use std::convert::AsRef;
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

use shellexpand::tilde;

pub fn enumerate_desktop_files<S>(sources: S) -> Vec<PathBuf>
where
    S: IntoIterator,
    S::Item: AsRef<Path>
{
   sources.into_iter()
       .filter_map(|source| {
           source.as_ref().to_str()
               .map(|path| tilde(path).into_owned())
               .and_then(|path| read_dir(path).ok())
       })
       .map(|d| d
           .filter_map(|e| e.ok()
               .map(|e| e.path())))
       .flatten()
       .collect::<Vec<PathBuf>>()
}

pub fn get_file_properties<P: AsRef<Path>>(filename: P) -> HashMap<&'static str, String> {
    let contents = read_to_string(filename).unwrap();
    println!("contents: {}", contents);

    parser::Parser::new("abcd");
    
    HashMap::new()
}