//! lsapp, List applications by scanning .desktop files
//!
//! Use `lsapp` to scan .desktop files and customize their display. Useful for creating a
//! simple program launcher by combining with fzf/skim

mod parser;

use std::str::FromStr;

use clap::clap_app;
use color_eyre::{Report, Result};
use eyre::WrapErr;
use thiserror::Error;

const DEFAULT_SOURCES: &[&'static str] = &[
    "/usr/share/applications",
    "/usr/local/share/applications",
    "~/.local/share/applications",
];

#[derive(Debug, Clone, Copy)]
enum Column<'a> {
    Name { lang: Option<&'a str> },
    Comment { lang: Option<&'a str> },
    Path,
    Filename { with_ext: bool },
    Categories,
    Icon,
}

impl<'a> FromStr for Column<'a> {
    type Err = Report;

    fn from_str(s: &str) -> Result<Column<'a>> {
        match s.to_lowercase().as_str() {
            "name" => Ok(Column::Name { lang: None }),
            "comment" => Ok(Column::Comment { lang: None }),
            "path" => Ok(Column::Path),
            "filename" => Ok(Column::Filename { with_ext: false }),
            "categories" => Ok(Column::Categories),
            "icon" => Ok(Column::Icon),
            _ => Err(AppError::InvalidColumn(s.into()).into()),
        }
    }
}

#[derive(Debug)]
enum Separator {
    Comma,
    Tab,
    Spaces,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("unsupported column type `{0}`")]
    InvalidColumn(String),

    #[error("{0}")]
    ArgError(clap::Error)
}

fn main() -> Result<()> {
    let matches = clap_app!(lsapp =>
        (version: "0.1")
        (author: "Carson Myers <carson@myers.se>")
        (about: "List installed applications scanned from .desktop files")
        (@arg sources: -S --sources +takes_value +multiple +use_delimiter
            env("LSAPP_SOURCES")
            default_value(&DEFAULT_SOURCES.join(","))
            "Source directories for application .desktop files")
        (@arg column: -d --data +takes_value +multiple +use_delimiter
            possible_values(&["name", "comment", "path", "filename", "categories", "icon"])
            default_value("name,comment,path")
            "Columns of data to include in the output")
        (@arg lang: -l --lang +takes_value
            "Language to use for name and comment, if available")
        (@arg ext: -x --("with-ext") "Includes extension in filename")
        (@arg comma: -c --comma conflicts_with_all(&["tab", "spaces"])
            "Separate columns with commas")
        (@arg tab: -t --tab conflicts_with_all(&["comma", "spaces"])
            "Separate columns with tabs")
        (@arg spaces: -s --spaces conflicts_with_all(&["comma", "tab"])
            "Separate columns with spaces as padding")
        (@arg quote: -q --quote "Quote values in columns")
    ).get_matches();

    let lang = matches.value_of("lang");
    let with_ext = matches.is_present("ext");

    let _columns = matches.values_of_t("column")
        .map_err(|err| AppError::ArgError(err))?
        .iter()
        .map(|v| match v {
            Column::Name { .. } => Column::Name { lang },
            Column::Comment { .. } => Column::Comment { lang },
            Column::Filename { .. } => Column::Filename { with_ext },
            _ => v.to_owned(),
        })
        .collect::<Vec<Column>>();

    let _separator = if matches.is_present("comma") {
        Separator::Comma
    } else if matches.is_present("tab") {
        Separator::Tab
    } else if matches.is_present("spaces") {
        Separator::Spaces
    } else {
        Separator::Tab
    };

    let sources = matches.values_of("sources")
        .map_or(vec![], |s| s.collect::<Vec<&str>>());
    let files = lsapp::enumerate_desktop_files(sources);
    for file in files {
        lsapp::get_file_properties(file);
    }

    Ok(())
}
