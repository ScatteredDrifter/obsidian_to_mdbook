// internal imports
pub mod structures;
pub mod config_parser;
pub mod settings;

use settings::{PATH_DEST, PATH_SOURCE, PATH_SUMMARY, PRINT_DEBUG, REQUEST_PATHS};
use structures::{string_to_fileextension, CollectedPaths, Config, ConfigType, Directory, FileExtension};
use config_parser::{parse_configuration,print_config};

// external import
use std::ffi::OsStr;
use std::error::Error;
use std::io::{self, BufReader, Write};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf,};


fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Insert a given path to traverse its directory and all contained files and directories");

    let configurations = wrapper_parse_config()?;

    if settings::PRINT_DEBUG{
        print_config(&configurations);  
    }
    // filtering out configuration that handle excluded_dirs 
    //IMPORTANT: assuming multiple may exist
    let filtered_dirs: Vec<String> = configurations
    .iter()
    .filter_map(|config| match config.conf_type {
       ConfigType::ExcludedPaths => Some(config.collection_of_options.clone()),
       _ => None,
    })
    .flatten() // reducing to one vector
    .collect();

    let paths: CollectedPaths = request_paths();
    let root_path = paths.root_dir;
    let save_path = paths.dest_file;
    let copy_directory = paths.dest_dir;
    if PRINT_DEBUG{
        println!("found following paths:\nroot:{}\ndest:{}\nsummary:{}\n",root_path.display(),copy_directory.display(),save_path.display())
    }

    // display_folder(&file_path);
    let parsed_dir = collect_dir_structure(
        &root_path,
        &filtered_dirs,
        &copy_directory,
        &root_path,
        );

    match parsed_dir {
        Ok(dir) => {
            visualize_directory(&dir,Some(1));
            let presentation:String = create_book_summary(&dir);
            // println!("{}",presentation);
            match save_to_file(&save_path, presentation) {
                Ok(_) => (),
                Err(error) => println!("{error}")
            }

            // COPYING FILES to new destination
            println!("copying files to destination: {}",&save_path.display());
            copy_directory_to_dest(&dir);
            println!("done copying files, update mdbook accordingly!")


        },
        Err(err) => println!("error {}",err)
    }
    Ok(())
}

fn request_paths() -> CollectedPaths {

    println!("collecting");
    if REQUEST_PATHS{
        CollectedPaths{
            root_dir: enforce_filepath(request_filepath),
            dest_dir: enforce_filepath(request_copy_path),
            dest_file: enforce_filepath(request_save_file)
        }
    } else {
        CollectedPaths{ 
            root_dir: PathBuf::from(PATH_SOURCE),
            dest_dir: PathBuf::from(PATH_DEST),
            dest_file: PathBuf::from(PATH_SUMMARY)
        }
    }

}

/// takes directory and copies it - recursively - to new destination
fn copy_directory_to_dest(base_dir:&Directory) -> () {

    // create directory first 
    let dest_dir = &base_dir.dest_path;
    if !dest_dir.exists() { 
        // does not exist, creating 
        // FIXME improved error handling
        let result_creation = fs::create_dir_all(dest_dir);
    }
    // copying files over from current directory
    for file in &base_dir.files{
        let file_copy_result = fs::copy(&file.original_path, &file.dest_path);
    }

    // once all have been copied, traverse to next directory 
    for directory in &base_dir.sub_directories{
        copy_directory_to_dest(&directory);
    }
    



}

/// opens and converts file to vector of configurations, or returns error
/// 
fn wrapper_parse_config() -> Result<Vec<Config>, Box<dyn Error>>{

    let path:PathBuf = PathBuf::from("/home/evelyn/Nextcloud/Notes/webpage_config.md");
    let file_reader = read_from_file(&path)?;
    parse_configuration(file_reader)
}

fn contains_excluded_path(path: &Path,exclusion: &Vec<String>) -> bool {
    for component in path.components(){
        if let Some(component_str) = component.as_os_str().to_str(){
            if exclusion.contains(&component_str.to_string()){
                return true
            }
        }
    }
    return false
}

/// takes Directory checks whether any .md file is contained in top-level folder 
/// returns True if one was found 
/// false otherwise
fn contains_md_file(directory:&Directory) -> bool { 
    for file in &directory.files{
        let extension  = &file.extension;
        match extension{
            FileExtension::Markdown=> return true,
            _ => {}
        };
    }
    return false;


}

///cuts path up to root of path traversed 
/// EXAMPLE:
/// /home/user/root_dir/dir1/dir2/test.md --> /dir1/dir2/test.md
fn remove_path_prefix(path:&PathBuf,old_path:&PathBuf) -> Result<PathBuf, Box<dyn Error>>{
    if path.starts_with(old_path){
        let shortened_path =  path.strip_prefix(old_path)
        .and_then(|new_path| Ok(new_path.to_path_buf() ) )?;
        Ok(shortened_path)
    } else {
       return  Err("prefix could not be removed from path: \n{path.to_display()}".into());
    }
}

/// @param
/// trimmed_base_path: denotes relative path from dest root path
/// dest_root_path: denotes root path to attached base_path onto
/// 
/// EXAMPLE: 
/// dest_root_path: /home/user/target_dir
/// trimmed_base_path: /subdir1/subdir2/target.md
/// returns /home/user/target_dir/subdir1/subdir2/target.md
fn create_dest_path(trimmed_base_path:&PathBuf,dest_root_path:&PathBuf) -> PathBuf{
    dest_root_path.join(trimmed_base_path)
}

/// receives directory, creates recursive structure as Directory struc, 
/// FIXME reduce complexity, refactor to collection of functions
fn collect_dir_structure(
    base_directory:&PathBuf,
    excluded_dirs:&Vec<String>,
    dest_path:&PathBuf,
    root_path:&PathBuf) -> Result<structures::Directory,Box<dyn std::error::Error>> {  
    // traversing the given Directory extracting information per subdir
    // assumes a correct path provided
    let parsed_path = Path::new(&base_directory).to_path_buf();

    let trimmed_dir_path = remove_path_prefix(&parsed_path.to_path_buf(), &root_path)?;
    let destination_path =create_dest_path(&trimmed_dir_path, &dest_path);
    // initializing object for given directory
    let mut current_dir: structures::Directory = structures::Directory{
        name:base_directory.file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_string(),
        path: base_directory.clone(),
        dest_path: destination_path,
        relative_path: trimmed_dir_path,
        sub_directories:Vec::new(),
        files: Vec::new()
    };

    let dirs = fs::read_dir(parsed_path)?;
    for entry in dirs{
        // traversing each entry
        let directory = entry?;
        let file_path = directory.path();
        if contains_excluded_path(&file_path.as_path(), &excluded_dirs){
            continue;
        }

        // in case a directory is found, we add those to our structure at the end 

        if file_path.is_dir() {
            match collect_dir_structure(&file_path,excluded_dirs,dest_path,root_path) {
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
            let as_file_extension = string_to_fileextension(&extension);

            let name:String = file_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned();

            let trimmed_path = remove_path_prefix(&file_path,&root_path)?;
            let destination_path_file =create_dest_path( &trimmed_path,&dest_path);
            current_dir.files.push(structures::FileData 
                {
                    name: name,
                    original_path: file_path,
                    dest_path: destination_path_file,
                    relative_path: trimmed_path,
                    extension: as_file_extension, 
                }
            );
        };
    }
    return Ok(current_dir);
}

/// visualizes supplied directory data structure 
/// prints each entry with files at given depth
fn visualize_directory(given_directory:&structures::Directory,indent:Option<usize>) -> () { 
    let indent = indent.unwrap_or(0);

    //  return information from active directory 
    let indentation:String = std::iter::repeat(" ").take(indent).collect();
    print!("{}|- [[{}]] :newpath {}\n"," ".repeat(indent-1),given_directory.name,given_directory.dest_path.display());

    for file in &given_directory.files {
        //  printing each file in same directory
        print!("{}|\n",indentation);
        print!("{}|-{}: newpath {} \n",indentation,file.name,file.dest_path.display());
    }
    for folder in &given_directory.sub_directories {
        //  print directory, increase indentation
        visualize_directory(&folder, Some(indent+1));
        }
}

/// converts given Directory instance to string for mdbook
/// wrapper for extract_file_representation_from_dir
/// uses structure for SUMMARY.md for mdbook
fn create_book_summary(directory_data:&structures::Directory) -> String {

    let directory_as_string:String = extract_file_representation_from_dir(&directory_data, 0);
    // print!("{directory_as_string}");
    let basic_formatting:String = format!("# SUMMARY.MD Structure\n\n{} ",directory_as_string);
    // for entry in directory_data.files
    return basic_formatting;
}

/// traverses Directory instance, converts to string complying for summary of mdbooks
/// IMPORTANT: Conceptualized as _recursive function_
fn extract_file_representation_from_dir(active_dir:&structures::Directory,depth:usize) -> String {

    let mut dir_as_string:String = String::new();

    // traversing and processing the active directory
    let stringified_dir: String = stringify_directory(&active_dir, depth);

    dir_as_string.push_str(&stringified_dir);

    // traversing all subsequent directories
    for directory in &active_dir.sub_directories {
        let dir_string = extract_file_representation_from_dir(&directory, depth+1);
        dir_as_string.push_str(&dir_string);
    }

    return dir_as_string
}

/// converts a Directory to string representation of its files 
/// depth denotes depth of headline to set -> indentation
fn stringify_directory(dir:&structures::Directory,depth:usize) -> String {
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
    // only pushing headline if the folder is not empty!
    if contains_md_file(dir){ 
        resulting_string.push_str(&headline);
    };

    for file in  &dir.files{
        // skipping if extension is mismatching
        let file_extension = &file.extension;
        match file.extension {
            FileExtension::Markdown => {
                let file_link:String = format!("{} - [{}]({})\n", " ".repeat(depth), file.name,file.relative_path.display());
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

fn request_filepath() -> Result<PathBuf,Box<dyn Error>> { 

    println!("\nenter directory to check and traverse");
    request_valid_path(false,false)
}

fn request_copy_path() -> Result<PathBuf, Box<dyn Error>> {
    println!("\nenter a path to copy content to");
    request_valid_path(false,false)
}

/// requests path for storing file 
/// prints "enter a filepath for {prompt}" 
fn request_save_file() -> Result<PathBuf,Box<dyn Error>> { 
    println!("\nenter a path to save to");
    request_valid_path(true,false)
}

fn request_valid_path(is_file:bool,is_unique:bool) -> Result<PathBuf,Box<dyn std::error::Error>> { 

    // println!(":{prompt}");

    // cleaning cache
    io::stdout().flush()?;
    let mut given_path = String::new();
    //  basically taking reference to this mutable object to allow adding the information from the io-stream directly!
    io::stdin().read_line(&mut given_path)?;

    let trimmed_path  =given_path.trim();
    
    // testing whether valid path was given
    let valid_path = PathBuf::from(trimmed_path);
    println!("{}",valid_path.display());

    if valid_path.is_dir() && !is_file{
        return Ok(valid_path)
    }
    if  valid_path.is_file() && is_file && is_unique {
        return Err("provided path exists already".into())
    }
    if is_file && !valid_path.exists(){
        return Ok(valid_path)
    } else {
        return Err("no valid path given".into())
    }

}

    // let added_extension_path = format!("{}Summary.md",trimmed_path);


fn save_to_file(file_path: &PathBuf, content: String) -> Result<(), Box<dyn std::error::Error>> {
        // Open the file in write mode, creating it if it doesn't exist
        let mut file = File::create(file_path)?;

        // Write the content to the file
        file.write_all(content.as_bytes())?;

        // Flush the file to ensure all data is written
        file.flush()?;

        Ok(())
}

fn read_from_file(file_path:&PathBuf) -> Result<BufReader<File>,Box<dyn std::error::Error>> {

    let file = File::open(file_path)?;

    let reader = BufReader::new(file);
    Ok(reader)
}
