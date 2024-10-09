/// holds several structs that are necessary to interact and represent data
/// 

// external imports
use std::path::{Path,PathBuf};

/// denotes a directory holding information about it 
/// contains all sub-dirs and files as vector 
pub struct Directory {

    pub path: PathBuf,
    pub name: String,
    pub dest_path: PathBuf,
    pub relative_path: PathBuf,
    pub sub_directories:Vec<Directory>,
    pub files:Vec<FileData>,
}


/// denotes a file and its associated infomrmation
/// 
pub struct FileData {
    pub original_path: PathBuf,
    pub dest_path: PathBuf,
    pub relative_path: PathBuf,
    pub extension: FileExtension,
    pub name:String
}

pub enum FileExtension {
    Markdown,
    Html,
    Image,
    Pdf,
    Unknown

}

pub struct CollectedPaths {
    pub root_dir: PathBuf,
    pub dest_dir: PathBuf,
    pub dest_file: PathBuf
}

/// FIXME --> Unkown is rather ambigous and prone to produce errors 
pub fn fileextension_to_string(extension:&FileExtension) -> String {
    match extension{
        FileExtension::Html => ".html".to_string(),
        FileExtension::Image => ".jpg".to_string(),
        FileExtension::Markdown => ".md".to_string(),
        FileExtension::Pdf => ".pdf".to_string(),
        FileExtension::Unknown => ".unknown".to_string(),
    }
}

/// takes extension as string and converts to FileExtension Struct
/// attention: each string is prefixed with a "." and has to be 
/// FIXME --> Unkown is rather ambigous and prone to produce errors 
pub fn string_to_fileextension(value:&String) -> FileExtension { 
    match value.as_str() {
        "md" => FileExtension::Markdown,
        "pdf" => FileExtension::Pdf,
        "html" => FileExtension::Html,
        "jpg" => FileExtension::Image,
        "png" => FileExtension::Image,
        _ => FileExtension::Unknown

    }
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