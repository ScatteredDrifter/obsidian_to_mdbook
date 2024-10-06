/// contains logic to provide a parser for configurations set in  obsidian-vault
/// 
/// the parser takes a path to a viable configuration-file that complies to the following structure: 
/// 
/// **the following is omitted** and not parsed:
/// - "---"
/// - "# . * "
/// - "date-*"
/// - "anchored.*"
/// - "\n"
/// 
/// a valid configuration ought to follow the following structure
/// 
/// starting collection of params with:
/// -> conf-start:ConfigType
/// followed by a list of values:
/// - param1
/// - param2 
/// ...
/// **closed by** given string:
/// -> conf-end:
/// 
/// Further everything after "--END-OF-CONFIG--" will not be read and skipped
/// example can be found in /doc

// internal imports
use crate::structures::{Config,ConfigType};

// external imports
use std::{error::Error, io::BufReader};
use std::fs::File;
use std::io::BufRead;
use regex::Regex;


//  CONFIG-CONSTANTS
//  change accordingly:

const CONFIG_START: &str = "conf-start:";
const CONFIG_END: &str =  "conf-end:";
const CONF_EXCLUDED_FILES: &str = "excluded_directories";
const CONF_PREFIXES: &str = "prefixes_for_headlines";

/// --- 
/// CORE FUNCTIONS
/// ---

/// FIXME naming lol
/// creates Config from given param
/// if no conf-end was received --> will return error, signaling wrong formatting
fn assemble_single_config(config_type:ConfigType,param_vec:Vec<String>) -> Result<Config,Box<dyn Error>> {

    let resulting_params: Vec<String> = param_vec
                            .iter()
                            .take_while(| &val| val != CONFIG_END)
                            .map(|val| val.to_string().replace("- ", ""))
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
fn vec_to_config(config_as_list:Vec<String>) -> Result<Vec<Config>,Box<dyn Error>> {

    // iterate through vector
    let regex_start = Regex::new(r"conf-start:.*").unwrap();
    let regex_end = Regex::new(r"conf-end").unwrap();

    let mut collection: Vec<Config> = Vec::new();
    let mut conf_params: Vec<String>;
    let mut found_start: bool = false;

    for entry in &config_as_list{ 

        if regex_start.is_match(entry.as_str()){
            conf_params = Vec::new();
            found_start = true;
            println!("found start with {entry}");
            // extracting type from string: 
            let type_as_string = entry.replace(CONFIG_START, "");
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

pub fn parse_configuration(file_buffer:BufReader<File>) -> Result<Vec<Config>,Box<dyn Error>> { 

    let blacklist = Regex::new(r"---.*|#.*|date-*|anchored.*|>.*|^\s*$").unwrap();
    let filtered_config: Vec<String> = file_buffer.lines()
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

/// ---- 
/// HELPER FUNCTIONS
/// ----

pub fn print_config(configs: &Vec<Config>) -> () { 
    for config in configs{

        let as_string = match config.conf_type{
            ConfigType::ExcludedPaths => "Excluded paths",
            ConfigType::PrefixHeadline => "headline prefixes"
        };
        println!("extracted config of type: {as_string}");
        for entry in &config.collection_of_options{
            println!("-> {entry}");
        };
        println!();
    }
    println!();
}
