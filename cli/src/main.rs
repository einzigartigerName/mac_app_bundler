extern crate clap;

use app_bundler::*;
use app_bundler::ExitCode::*;

use clap::{App, Arg};
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

const ARG_BINARY: &str = "binary";
const ARG_ICON: &str = "icon";
const ARG_NAME: &str = "output";


/// Parse Arguments and create Data Struct
fn parse_args() -> Result<DataParsed, ExitCode> {
    let matches = App::new("AppBundler")
        .version("0.1.0")
        .about("Bundle your binary to a MacOS .app")

        .arg(Arg::with_name(ARG_BINARY)
            .short("b")
            .long("binary")
            .value_name("FILE")
            .help("Binary to bundle")
            .takes_value(true)
            .required(true))

        .arg(Arg::with_name(ARG_ICON)
            .short("i")
            .long("icon")
            .value_name("ICON")
            .help("Icon to use (.icns)")
            .takes_value(true))

        .arg(Arg::with_name(ARG_NAME)
            .short("o")
            .long("output")
            .value_name("NAME")
            .help("Output App (Default: binary name")
            .takes_value(true))

        .get_matches();

    let arg_icon = matches.value_of(ARG_ICON);
    let arg_name = matches.value_of(ARG_NAME);

    let binary = if let Ok(path) = PathBuf::from_str(matches.value_of(ARG_BINARY).unwrap()) {
        path
    } else {
        println!("Unable to find binary!");
        return Err(BinaryNotFound)
    };

    let icon = opt_path_from_opt_str(arg_icon);
    if let Some(ref val) = icon {
        if ! is_icns(val) {
            eprintln!("Icon not a .icns file!");
            return Err(WrongFileFormat)
        }
    }

    let name = opt_path_from_opt_str(arg_name);

    Ok(DataParsed {
        name,
        binary,
        icon,
    })
}

/// Convert Option<&str> to Option<PathBuf>
fn opt_path_from_opt_str(input: Option<&str>) -> Option<PathBuf> {
    if let Some(val) = input {
        if let Ok(path) = PathBuf::from_str(val) {
            Some(path)
        } else { None }
    } else { None }
}

fn main() {
    let data = match parse_args() {
        Ok(values) => values,
        Err(code) => {
            process::exit(code as i32)
        }
    };

    match bundle(&data) {
        Ok(()) => process::exit(Success as i32),
        Err(code) => process::exit(code as i32),
    }
}