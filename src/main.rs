extern crate rust_embed;
extern crate clap;

use clap::{App, Arg};
use rust_embed::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::process;
use std::fs::{File, create_dir, copy, create_dir_all};
use std::io::{ErrorKind, Error, Write};
use std::os::unix::fs::PermissionsExt;
use std::ffi::OsStr;

const ARG_BINARY: &str = "binary";
const ARG_ICON: &str = "icon";
const ARG_NAME: &str = "name";

const DIR_CONTENT: &str = "Contents";
const DIR_RESOURCES: &str = "Resources";
const DIR_MACOS: &str = "MacOS";

const FILE_LAUNCHER: &str = "launcher";
const FILE_PLIST: &str = "Info.plist";

const ICON_EXT: &str = "icns";

const SHELL_BANG: &str = "#! /bin/sh";
const EXEC_VAR: &str = "EXEC=";
const DIR_VAR: &str = "DIR=$(cd \"$(dirname \"$0\")\"; pwd)";
const EXEC_CMD: &str = "exec \"$DIR/$EXEC\"";

#[derive(RustEmbed)]
#[folder="assets/"]
struct Assets;

#[derive(Debug)]
struct DataParsed {
    name: Option<PathBuf>,
    binary: PathBuf,
    icon: Option<PathBuf>,
}

/// Parse Arguments and create Data Struct
fn parse_args() -> DataParsed {
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
            .short("n")
            .long("name")
            .value_name("NAME")
            .help("Name of your App (Default: binary name")
            .takes_value(true))

        .get_matches();

    let arg_icon = matches.value_of(ARG_ICON);
    let arg_name = matches.value_of(ARG_NAME);

    let binary = if let Ok(path) = PathBuf::from_str(matches.value_of(ARG_BINARY).unwrap()) {
        path
    } else {
        println!("Unable to find binary!");
        process::exit(1);
    };

    let icon = opt_path_from_opt_str(arg_icon);
    if let Some(ref val) = icon {
        if val.extension().and_then(OsStr::to_str) != Some(ICON_EXT){
            eprintln!("Icon not a .icns file!");
            process::exit(1);
        }
    }

    let name = opt_path_from_opt_str(arg_name);

    DataParsed {
        name,
        binary,
        icon,
    }
}

/// Convert Option<&str> to Option<PathBuf>
fn opt_path_from_opt_str(input: Option<&str>) -> Option<PathBuf> {
    if let Some(val) = input {
        if let Ok(path) = PathBuf::from_str(val) {
            Some(path)
        } else { None }
    } else { None }
}

/// Create Content for the Launch Script
fn create_launch_context(binary: &str) -> String {
    format!(
        "{}\n\n{}\"{}\"\n{}\n{}\n",
        SHELL_BANG,
        EXEC_VAR,
        binary,
        DIR_VAR,
        EXEC_CMD
    )
}

/// Create Info.plist
fn create_plist(icon: Option<String>) -> String {
    let plist = Assets::get(FILE_PLIST).unwrap();

    let mut content = String::new();
    content.push_str(
        std::str::from_utf8(&plist).unwrap()
    );

    if let Some(value) = icon {
        content.push_str("\t\t<key>CFBundleIconFile</key>\n");
        content.push_str(
            &*format!("\t\t<string>{}</string>\n", value)
        );
    }
    content.push_str("\t</dict>\n</plist>");
    content
}

/// Creates Directory Structure
fn create_file_structure(location: &mut PathBuf) -> Result<(), Error> {
    location.set_extension("app");

    if location.exists() {
        return Err(ErrorKind::AlreadyExists.into())
    }

    // AppBundle.app
    create_dir_all(&location)?;

    // AppBundle.app/Contents
    location.push(DIR_CONTENT);
    create_dir(&location)?;

    // AppBundle.app/Contents/MacOS
    location.push(DIR_MACOS);
    create_dir(&location)?;

    // AppBundle.app/Contents/Resources
    let _ = location.pop();
    location.push(DIR_RESOURCES);
    create_dir(&location)?;

    location.pop();
    Ok(())
}

/// MAIN
fn main() {
    if ! cfg!(unix) {
        eprintln!("Error: Not on a unix-like OS system!\n Please use on a Unix-like OS");
        process::exit(1);
    }

    /* Parse Args */
    let data = parse_args();

    /* Validate Data */
    if ! data.binary.exists() {
        eprintln!("File \"{}\" does not exist!", data.binary.to_str().unwrap());
        return;
    }

    if let Some(icons) = &data.icon {
        if ! icons.exists() {
            eprintln!("File \"{}\" does not exist!", icons.to_str().unwrap());
            return;
        }
    }

    /* Var for later use */
    let binary_name = String::from(
        data.binary
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );

    let icon_name = if let Some(ref path) = data.icon {
        Some(String::from(path.file_name().unwrap().to_string_lossy()))
    } else {
        None
    };

    let mut app_dir = if let Some(path) = data.name {
        path
    } else {
        let mut local = PathBuf::new();
        local.push(&binary_name);
        local
    };

    /* Create Directory Structure */
    match create_file_structure(&mut app_dir) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("create {}", err);
            process::exit(1)
        }
    };

    /* Copy Binary, Icon and write Info.plist */
    println!("Contents Location: {}", app_dir.to_str().unwrap());

    // create Info.plist
    app_dir.push(FILE_PLIST);
    let mut plist = match File::create(&app_dir) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to create {}: {}", FILE_PLIST, err);
            process::exit(1)
        }
    };

    match plist.write_all(create_plist(icon_name).as_bytes()) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to create {}: {}", FILE_PLIST, err);
            process::exit(1)
        }
    }
    app_dir.pop();

    // copy executable
    app_dir.push(DIR_MACOS);
    app_dir.push(&binary_name);
    match copy(&data.binary, &app_dir) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Unable to move executable: {}", err);
            process::exit(1);
        }
    }

    // create launch script
    app_dir.pop();
    app_dir.push(FILE_LAUNCHER);
    let mut launcher = match File::create(&app_dir) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to create launch script: {}", err);
            process::exit(1)
        }
    };
    match launcher.write_all(create_launch_context(&binary_name).as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Unable to create launch script: {}", err);
            process::exit(1);
        }
    }

    let metadata = launcher.metadata().unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o554);
    match launcher.set_permissions(permissions) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Unable to create launch script: {}", err);
            process::exit(1)
        }
    }

    // copy app icon
    if let Some(icon) = &data.icon {
        app_dir.pop();
        app_dir.pop();
        app_dir.push(DIR_RESOURCES);
        app_dir.push(icon.file_name().unwrap().to_str().unwrap());

        match copy(icon, &app_dir) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Unable to copy icons: {}", err);
            }
        }
    }
}