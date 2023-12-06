use std::path::{Path, PathBuf};
use crate::markdown;

use anyhow::Result;

// Overwrites files in B with the changes from A
fn sync_a2b() {}

pub fn move_published<P: AsRef<Path>>(src: P, dst: P) -> Result<()> {  
    let pub_files = markdown::public_files(&src)?;

    pub_files.into_iter().for_each(|(ref f, header)| {

        if header.path.is_some() {
            let filepath = PathBuf::from(header.path.unwrap());
            let base = dst.as_ref();
            println!("{:?} => {:?}", &base, base.components());
            if let Some(without_file) = base.parent() {
                std::fs::create_dir_all(&without_file).unwrap();
            }
            let base = base.join(filepath);
            println!("{:?} => {:?}", &f, &base);

            match std::fs::copy(f, &base) {
                Ok(_) => {}
                //Err(_) => panic!("{:?}", &base)
                Err(_) => {}
            };
        } else {
            let base = dst.as_ref().join("blog");
            let fname = f.file_name().unwrap();
            std::fs::create_dir_all(&base).unwrap();
            let end = base.join(fname);
            println!("{:?} => {:?}", &f, &end);
            std::fs::copy(f, end).unwrap();
        }
    });

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_move_published() {
        move_published("./tmp", "../blog-root/content").unwrap();
    }
}
