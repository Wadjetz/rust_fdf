extern crate ring;
use ring::digest;

extern crate walkdir;
use walkdir::WalkDir;
use walkdir::DirEntry;

use std::fs::File;
use std::fs::Metadata;
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;
use std::collections::HashMap;

extern crate clap;
use clap::{Arg, App, AppSettings};

#[derive(Debug)]
pub struct FileIndex {
    pub dir_entry: DirEntry,
    pub metadata: Option<Metadata>
}

impl FileIndex {
    pub fn new(dir_entry: DirEntry) -> Self {
        //let metadata = std::fs::metadata(dir_entry.path()).ok();
        FileIndex { dir_entry, metadata: None }
    }
}

fn main() {
    let matches = App::new("Find duplicated files")
        .version("0.0.1")
        .author("Wadjetz")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(Arg::with_name("directory")
            .short("d")
            .long("dir")
            .takes_value(true)
            .required(true)
            .multiple(true)
            .validator(|value| {
                let path = Path::new(&value);
                if path.exists() {
                    if path.is_dir() {
                        Ok(())
                    } else {
                        Err("is not a directory".to_owned())
                    }
                } else {
                    Err("Directory is not existe".to_owned())
                }
            })
            .help("test")
        ).get_matches();
    
    if let Some(directories) = matches.values_of("directory") {
        let index = directories
            .map(Path::new)
            .flat_map(get_directories)
            .fold(HashMap::new(), |mut acc, file_index| {
                let hash = get_hash(file_index.dir_entry.path());
                {
                    let entry = acc.entry(hash).or_insert(Vec::new());
                    entry.push(file_index);
                }
                acc
            });
        
        for (_key, value) in index.iter().filter(|key| key.1.len() > 1) {
            println!("Which file to delete ? select the index, or other character for pass");
            for (i, file) in value.iter().enumerate() {
                println!("{} {:?}", i, file.dir_entry.path());
            }
            if let Ok(response) = get_response() {
                println!("Number selected {}", response);
                match response.trim().parse::<usize>() {
                    Ok(number) => {
                        if let Some(f) = value.get(number) {
                            delete_file(f.dir_entry.path())
                        } else {
                            println!("Wrong selection");
                        }
                    },
                    Err(e) => {
                        println!("Error parsing {}", e);
                    }
                }
            }
        }
    };
}

pub fn get_directories(path: &Path) -> Vec<FileIndex> {
    WalkDir::new(path).into_iter()
        .filter_map(|r| r.ok())
        .filter(|f| f.file_type().is_file())
        .map(FileIndex::new)
        .collect()
}

pub fn get_response() -> Result<String, std::io::Error> {
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut stdin_buffer = String::new();
    stdin.read_line(&mut stdin_buffer)
         .map(|_| stdin_buffer)
}

pub fn delete_file(path: &Path) {
    match std::fs::remove_file(path.as_os_str()) {
        Ok(_) => println!("{:?} deleted", path),
        Err(e) => println!("Can't delete file {}", e)
    }
}

pub fn hash(content: &[u8]) -> String {
    let hasher = digest::digest(&digest::SHA1, content);
    let hash = hasher.as_ref();
    let vec_hash = Vec::from(hash);
    vec_hash.iter().map(|b| format!("{:x}", b)).collect()
}

pub fn file_content(file: &File) -> Result<Vec<u8>, std::io::Error> {
    let mut read_buffer = BufReader::new(file);
    let mut buffer = Vec::new();
    read_buffer.read_to_end(&mut buffer)?;
    Ok(buffer)
}

pub fn get_hash(path: &Path) -> String {
    let file = File::open(path).expect(&format!("File not found {:?}", path));
    let content = file_content(&file).unwrap();
    hash(&content)
}
