/// holds several structs that are necessary to interact and represent data
/// 

// external imports
use std::path::{Path,PathBuf};

/// denotes a directory holding information about it 
/// contains all sub-dirs and files as vector 
pub struct Directory {

    pub path: PathBuf,
    pub name: String,
    pub parent: Option<Box<PathBuf>>,
    pub sub_directories:Vec<Directory>,
    pub files:Vec<FileData>,
}


/// denotes a file and its associated infomrmation
/// 
pub struct FileData {
    pub path: PathBuf,
    pub extension: String,
    pub name:String
}

// Structures for Config parsing
pub struct Config {
    pub conf_type: ConfigType,
    pub collection_of_options: Vec<String>
}

pub enum ConfigType {
    ExcludedPaths,
    PrefixHeadline,
}