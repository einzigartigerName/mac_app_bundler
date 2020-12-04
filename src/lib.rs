extern crate rust_embed;

use rust_embed::*;
use std::path::PathBuf;
use std::fs::{create_dir_all, create_dir, copy, File};
use std::os::unix::fs::PermissionsExt;
use std::io::{ErrorKind, Error, Write};
use crate::ExitCode::*;

const DIR_CONTENT: &str = "Contents";
const DIR_RESOURCES: &str = "Resources";
const DIR_MACOS: &str = "MacOS";

const FILE_LAUNCHER: &str = "launcher";
const FILE_PLIST: &str = "Info.plist";

pub const ICON_EXT: &str = "icns";

const SHELL_BANG: &str = "#! /bin/sh";
const EXEC_VAR: &str = "EXEC=";
const DIR_VAR: &str = "DIR=$(cd \"$(dirname \"$0\")\"; pwd)";
const EXEC_CMD: &str = "exec \"$DIR/$EXEC\"";

#[derive(RustEmbed)]
#[folder="assets/"]
struct Assets;

/// Exit Error Codes
/// Success: 0
/// Single digit (1-9): Creating/Writing/Copy Files
/// Beginning with 1 (10-19): OS type Error
/// Beginning with 2 (20-29): Interactive Errors (Gui exclusive)
/// Beginning with 3 (30-39): CLI Errors (CLI exclusive)
#[derive(Debug)]
pub enum ExitCode {
    Success = 0,
    BinaryNotFound = 1,
    IconNotFound = 2,
    UnableToCreate = 3,
    UnableToWrite = 4,
    UnableToCopy = 5,
    ChangePermission = 6,
    WrongFileFormat = 7,
    NotUnixSystem = 10,
    FileDialogError = 20,
}

impl Default for ExitCode {
    fn default() -> Self {
        Success
    }
}

#[derive(Debug, Default)]
pub struct DataParsed {
    pub name: Option<PathBuf>,
    pub binary: PathBuf,
    pub icon: Option<PathBuf>,
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

/// Create App Bundle
pub fn bundle(data: &DataParsed) -> Result<(), ExitCode> {
    if ! cfg!(unix) {
        eprintln!("Error: Not on a unix-like OS!");
        return Err(NotUnixSystem)
    }

    /* Validate Data */
    if ! data.binary.exists() {
        eprintln!("File \"{}\" does not exist!", data.binary.to_str().unwrap());
        return Err(BinaryNotFound)
    }

    if let Some(icons) = &data.icon {
        if ! icons.exists() {
            eprintln!("File \"{}\" does not exist!", icons.to_str().unwrap());
            return Err(IconNotFound)
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

    let mut app_dir = if let Some(ref path) = data.name {
        path.to_owned()
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
            return Err(UnableToCreate)
        }
    };

    /* Copy Binary, Icon and write Info.plist */
    // create Info.plist
    app_dir.push(FILE_PLIST);
    let mut plist = match File::create(&app_dir) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to create {}: {}", FILE_PLIST, err);
            return Err(UnableToCreate)
        }
    };

    match plist.write_all(create_plist(icon_name).as_bytes()) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to write to {}: {}", FILE_PLIST, err);
            return Err(UnableToWrite)
        }
    }
    app_dir.pop();

    // copy executable
    app_dir.push(DIR_MACOS);
    app_dir.push(&binary_name);
    match copy(&data.binary, &app_dir) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Unable to copy executable: {}", err);
            return Err(UnableToCopy)
        }
    }

    // create launch script
    app_dir.pop();
    app_dir.push(FILE_LAUNCHER);
    let mut launcher = match File::create(&app_dir) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Unable to create launch script: {}", err);
            return Err(UnableToCreate)
        }
    };
    match launcher.write_all(create_launch_context(&binary_name).as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Unable to write launch script: {}", err);
            return Err(UnableToWrite)
        }
    }

    let metadata = launcher.metadata().unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o554);
    match launcher.set_permissions(permissions) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Unable to create launch script: {}", err);
            return Err(ChangePermission)
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
                return Err(UnableToCopy)
            }
        }
    }

    Ok(())
}