use anyhow::Result;
use fronma::parser::parse;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Error, Debug)]
pub enum MarkdownError {
    #[error("{0}: error validating frontmatter")]
    Frontmatter(String),
}

#[derive(Deserialize, Debug)]
pub struct Headers {
    pub publish: Option<bool>,
    pub title: Option<String>,
    pub path: Option<String>,
    pub date: Option<String>,
    pub tags: Option<Vec<String>>,
}

pub fn headers<T: AsRef<Path>>(fname: T) -> Result<Headers> {
    let text = fs::read_to_string(&fname)?;

    match parse::<Headers>(&text) {
        Ok(data) => {
            //println!("{:#?}", data);
            Ok(data.headers)
        }
        Err(e) => {
            //println!("{:#?}", e);
            Err(MarkdownError::Frontmatter(fname.as_ref().to_string_lossy().to_string()).into())
        }
    }
}

/// The basedir file should be where a directory which contains the markdown notes
pub fn public_files<T: AsRef<Path>>(basedir: T) -> Result<Vec<(PathBuf, Headers)>> {
    let fs = WalkDir::new(basedir)
        .into_iter()
        .filter_map(|e| {
            if e.as_ref().unwrap().file_type().is_file() {
                let headers = headers(e.as_ref().unwrap().path());
                match headers {
                    Ok(Headers {
                        publish: Some(true),
                        ..
                    }) => Some((e.as_ref().unwrap().path().to_path_buf(), headers.unwrap())),
                    Ok(_) => None,
                    Err(_) => None,
                }
            } else {
                None
            }
        })
        .collect();
    Ok(fs)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::dropbox;
    use std::path::PathBuf;

    ///TODO: Refactor tests to download files before
    #[tokio::test]
    async fn fmatter() {
        dotenv::dotenv().ok();
        let token = dropbox::fetch_sl_token_with_refresh().await.unwrap();
        let _file = dropbox::fetch_api_zip(&token, "/journal", "./tmp.zip")
            .await
            .unwrap();
        zip_extract::extract(
            &std::fs::File::open("./tmp.zip").unwrap(),
            &PathBuf::from("./tmp"),
            true,
        )
        .unwrap();
        let f = "./tmp/blog index.md";
        let _ = headers(f).unwrap();
    }
    //#[test]
    //fn test_public_files() {
    //    let fs = public_files("./tmp");
    //    println!("{:#?}", fs);
    //}
}
