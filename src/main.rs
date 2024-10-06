// internal imports
pub mod structures;

use std::error::Error;
// external import
use std::ffi::OsStr;
// use std::fmt::format;
use std::io::{self, BufRead, BufReader, Write};
// use std::thread::current;
// use std::fmt::Error;
// importing filesystem 
use std::{fs, option, result};
use std::fs::File;
use std::path::{Path, PathBuf};
use regex::Regex;
use structures::{Config, ConfigType};

// constants
// **FIXME** --> transport to config-parser module! 
const CONFIG_START: &str = "conf-start:";
const CONFIG_END: &str =  "conf-end:";
const CONF_EXCLUDED_FILES: &str = "excluded_directories";
const CONF_PREFIXES: &str = "prefixes_for_headlines";

fn print_config(configs: Vec<Config>) -> () { 
    for config in configs{

        let as_string = match config.conf_type{
            ConfigType::ExcludedPaths => "Excluded paths",
            ConfigType::PrefixHeadline => "headline prefixes"
        };
        println!("extracted config of type: {as_string}");
        for entry in config.collection_of_options{
            print!("{entry}");
        };
        println!();
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Insert a given path to traverse its directory and all contained files and directories");

    let configurations = parse_configuration()?;
    // DEBUG
    print_config(configurations);

    let file_path = enforce_filepath(request_filepath);
    let save_path:PathBuf = enforce_filepath(request_file_to_save_to);
        
    display_folder(&file_path);
    let parsed_dir = collect_dir_structure(file_path, &None);
    match parsed_dir {
        Ok(dir) => {
            unwrap_directory(&dir,Some(1));
            let presentation:String = create_book_summary(dir);
            println!("{}",presentation);
            match save_to_file(save_path, presentation) {
                Ok(content) => (),
                Err(error) => println!("{error}")
            }
        },
        Err(err) => println!("error {}",err)
    }

    Ok(())
}

/// DEBUG-FUNCTION
/// helps to display / traverse content of given path
/// prints accordingly
fn display_folder(proposed_path:&PathBuf) -> Result<(),Box<dyn std::error::Error>>  { 

    let parsed_path = Path::new(&proposed_path);
    let display_path = parsed_path.display();
    println!("traversing the following directory: {display_path}");

    // traversing every entry in given path
    for file in fs::read_dir(parsed_path)? { 
        let entry = file?; 
        let path = entry.path();
        // randomly gathering metadata about the objects themself
        let metadata = fs::metadata(&path)?;
        let last_modified = metadata.modified()?.elapsed()?.as_secs();

        println!(
            "Currently selected file is:{:?} and it was last modified on {:?} ",
            path.file_name().ok_or("no filename given")?,
            last_modified,
        );
    }   
    Ok(())
    }

fn collect_dir_structure(base_directory:PathBuf,parent_path:&Option<Box<PathBuf>>) -> Result<structures::Directory,Box<dyn std::error::Error>> {  
    // traversing the given Directory extracting information per subdir
    // assumes a correct path provided

    // initializing object for given directory
    let mut current_dir: structures::Directory = structures::Directory{
        name:base_directory.file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_string(),

        path: base_directory.clone(),
        parent: parent_path.clone(),
        sub_directories:Vec::new(),
        files: Vec::new()
    };

    let parsed_path = Path::new(&base_directory);
    let entries = fs::read_dir(parsed_path)?;
    for entry in entries{
        // traversing each entry
        let file = entry?;
        let file_path = file.path();

        // in case a directory is found, we add those to our structure at the end 

        if file_path.is_dir(){
            let new_base = Some(Box::new(base_directory.clone()));
            match collect_dir_structure(file_path.clone(),&new_base) {
                Ok(dir) => current_dir.sub_directories.push(dir),
                Err(error) => println!("error while processing sub_directory, with following error \n {error}"),
            };
        };


        // in case a file was found
        // storing file in new struct
        if file_path.is_file(){

            let extension = file_path.extension()
            .and_then(OsStr::to_str)
            .unwrap_or("")
            .to_string();

            let name:String = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned();
            current_dir.files.push(structures::FileData 
                {
                    name: name,
                    path: file_path,
                    extension: extension, 
                }
            );
        };
    }
    return Ok(current_dir);
}

/// visualizes supplied directory data structure 
/// prints each entry with files at given depth
fn unwrap_directory(given_directory:&structures::Directory,indent:Option<usize>) -> () { 
    let indent = indent.unwrap_or(0);

    //  return information from active directory 
    let display_path = given_directory.path.display();
    let indentation:String = std::iter::repeat(" ").take(indent).collect();
    print!("{}|- [[{}]]\n"," ".repeat(indent-1),given_directory.name);

    for file in &given_directory.files {
        //  printing each file in same directory
        print!("{}|\n",indentation);
        print!("{}|-{}\n",indentation,file.name);
    }
    for folder in &given_directory.sub_directories {
        //  print directory, increase indentation
        unwrap_directory(&folder, Some(indent+1));
        }
}

/// converts given Directory instance to string for mdbook
/// wrapper for extract_file_representation_from_dir
/// uses structure for SUMMARY.md for mdbook
fn create_book_summary(directory_data:structures::Directory) -> String {

    let directory_as_string:String = extract_file_representation_from_dir(&directory_data, 0);
    print!("{directory_as_string}");
    let basic_formatting:String = format!("SUMMARY.MD Structure\n\n{} ",directory_as_string);
    // for entry in directory_data.files
    return basic_formatting;
}

/// traverses Directory instance, converts to string complying for summary of mdbooks
/// IMPORTANT: Conceptualized as _recursive function_
fn extract_file_representation_from_dir(active_dir:&structures::Directory,depth:usize) -> String {

    let mut dir_as_string:String = String::new();

    // traversing and processing the active directory
    let stringified_dir: String = stringify_directory(&active_dir, depth, None);

    dir_as_string.push_str(&stringified_dir);

    // traversing all subsequent directories
    for directory in &active_dir.sub_directories {
        let dir_string = extract_file_representation_from_dir(&directory, depth+1);
        print!("{dir_string}\n");
        dir_as_string.push_str(&dir_string);
    }

    return dir_as_string
}

/// converts a Directory to string 
/// depth denotes depth of headline to set -> indentation
fn stringify_directory(dir:&structures::Directory,depth:usize, excluded_files:Option<Vec<String>>) -> String {
    // given a directory 
    // depth denotes depth of headline to set 
    // 

    // creating headline for given directory -> taking only its name
    let headline:String = format!(
        "{} {}\n",
        "#".repeat(depth),
        dir.name 
    );
    // traversing each file and directory
    let mut resulting_string = String::new();

    for file in  &dir.files{
        // skipping if extension is mismatching
        let file_extension = file.extension.as_str();
        // print!("extension of file: {file_extension}\n");
        match file.extension.as_str() {
            "md" => {
                print!("extension of file: {file_extension}\n");
                let file_link:String = format!("{} - [{}]({})\n", " ".repeat(depth), file.name,file.path.display());
                resulting_string.push_str(&file_link)
                },
            _ => {},
        }
    };
    return resulting_string;

    }

//  ------ 
//  ------HELPER FUNCTIONS------ 
//  ------ 

/// requesting user to provide valid path 
/// returns path or error 
fn request_filepath() -> Result<PathBuf,Box<dyn std::error::Error>> {

    println!("enter directory to check and traverse");

    // cleaning cash
    io::stdout().flush()?;
    let mut given_path = String::new();
    //  basically taking reference to this mutable object to allow adding the information from the io-stream directly!
    io::stdin().read_line(&mut given_path)?;

    let trimmed_path  =given_path.trim();
    
    // testing whether valid path was given
    let valid_path = PathBuf::from(trimmed_path);
    if valid_path.exists() && valid_path.is_dir(){
        // found valid input, continuing
        return Ok(valid_path)
    }
    Err("provided path was not valid, retry with a valid path again".into())

}

/// requests valid input for given function
/// does repeat input on errors -> guaranteeing return value to be PathBuf
fn enforce_filepath(function_to_enforce:fn() -> Result<PathBuf,Box<dyn std::error::Error>>) -> PathBuf {
    // wrapper for request_filepath, guarantees valid path to be returned

    let valid_path:PathBuf = loop {
        match function_to_enforce() {
            Ok(valid_path) => break valid_path,
            Err(error) => {
                println!("certain error was thrown:\n{}",error)

            }
        }
    };
    return valid_path
}

fn request_file_to_save_to() -> Result<PathBuf,Box<dyn std::error::Error>> { 

    println!("enter directory to save file to");

    // cleaning cash
    io::stdout().flush()?;
    let mut given_path = String::new();
    //  basically taking reference to this mutable object to allow adding the information from the io-stream directly!
    io::stdin().read_line(&mut given_path)?;

    let trimmed_path  =given_path.trim();
    let added_extension_path = format!("{}Summary.md",trimmed_path);
    
    // testing whether valid path was given
    let valid_path = PathBuf::from(added_extension_path);
    println!("{}",valid_path.display());
    
    if valid_path.is_file(){
        Err("provided path exists already".into())
    } 
    else {
        return Ok(valid_path)
    }



}

fn save_to_file(file_path: PathBuf, content: String) -> Result<(), Box<dyn std::error::Error>> {
        // Open the file in write mode, creating it if it doesn't exist
        let mut file = File::create(file_path)?;

        // Write the content to the file
        file.write_all(content.as_bytes())?;

        // Flush the file to ensure all data is written
        file.flush()?;

        Ok(())
}

fn read_from_file(file_path:PathBuf) -> Result<BufReader<File>,Box<dyn std::error::Error>> {

    let file = File::open(file_path)?;

    let reader = BufReader::new(file);
    Ok(reader)
}

/// FIXME naming lol
/// creates Config from given param
/// if no conf-end was received --> will return error, signaling wrong formatting
fn assemble_single_config(config_type:ConfigType,param_vec:Vec<String>) -> Result<Config,Box<dyn Error>> {

    let resulting_params: Vec<String> = param_vec
                            .iter()
                            .take_while(| &val| val != CONFIG_END)
                            .map(|val| val.to_string())
                            .collect();
    if resulting_params.is_empty() { 
        return Err("no params found before conf-end: --> typo?".into())
    };
    // processed correctly 
    Ok(Config{
        conf_type: config_type,
        collection_of_options: resulting_params
    })
}


/// fn construct_excluded_files(&filtered_config:Iterator<Item = <Result<>>) -> structures::ExcludedFiles {
/// with given iterator we search for "start" and end 
/// }
/// FIXME Improve structure of collecting 
fn vec_to_config(config_as_list:Vec<String>) -> Result<Vec<structures::Config>,Box<dyn Error>> {

    // iterate through vector
    let start_string = "conf-start:";
    let end_string = "conf-end:";
    let regex_start = Regex::new(r"conf-start:.*").unwrap();
    let regex_end = Regex::new(r"conf-end").unwrap();

    let mut collection: Vec<structures::Config> = Vec::new();
    let mut conf_params: Vec<String>;
    let mut found_start: bool = false;

    for entry in &config_as_list{ 

        if regex_start.is_match(entry.as_str()){
            conf_params = Vec::new();
            found_start = true;
            println!("found start with {entry}");
            // extracting type from string: 
            let type_as_string = entry.replace(start_string, "");
            let option_type = match type_as_string.as_str() { 
                CONF_EXCLUDED_FILES => ConfigType::ExcludedPaths,
                CONF_PREFIXES => ConfigType::PrefixHeadline,
                _ => return Err(format!("no matching config-param was supplied {type_as_string}").into()),
            };
            // gathering params for given part
            let rest_without_start: Vec<String> = config_as_list.split(|value| value == entry)
            .nth(1)
            .unwrap_or(&[])
            .to_vec();

            let resulting_confg = assemble_single_config(option_type, rest_without_start);
            match resulting_confg{
                Ok(config) => collection.push(config),
                Err(e) => return Err(format!("error parsing config\n {e}").into())
            }
        }

    }
    println!("finished parsing config!");
    Ok(collection)
}

fn parse_configuration() -> Result<Vec<Config>,Box<dyn Error>> { 

    let path:PathBuf = PathBuf::from("/home/evelyn/Nextcloud/Notes/webpage_config.md");
    let file_reader = read_from_file(path);

    let blacklist = Regex::new(r"---.*|#.*|date-*|anchored.*|>.*|^\s*$").unwrap();
    match file_reader {

        Err(val) => Err(val), 

        Ok(file_reader) => { 

            let filtered_config: Vec<String> = file_reader.lines()
                .filter_map(|line| match line { 
                    Ok(line) => { 
                        if !blacklist.is_match(&line) {
                            Some(line)
                        } else { 
                            None 
                        }
                    }
                    Err(e) => None,
                })
                .collect();
            // processing filtered config!
            let parsed_config = vec_to_config(filtered_config);
            match parsed_config{
                Ok(configurations) => return Ok(configurations),
                Err(e) => return Err(format!("error converting, see: {e}").into())
            };
        }
    }

}