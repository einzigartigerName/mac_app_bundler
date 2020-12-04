# MacOS App Bundler
Create an .app Bundle from a binary.
There is a CLI and a GUI Version for you to choose from.

## Build
To build the cli package
```shell script
git clone https://github.com/einzigartigername/mac_app_bundler
cd mac_app_bundler
cargo build --package cli
```
For the gui version use
```shell script
cargo build --package gui
```


## Install
For the cli tool:
```shell script
cargo install --path cli
```
For the gui tool:
```shell script
cargo install --path gui
```

## Usage
Minimal Requirement is a binary, the rest (Icon and App Name) is optional.
If no App Name is provided, the Output will have the binary name.

### Command Line Interface
Arguments - app-bundler-cli [OPTIONS] --binary FILE
* `-b` `--binary` Binary to Bundle
* `-i` `--icon` Icon to use (.icns Format)
* `-o` `--output` Output App (Default: binary name)
* `-h` `--help` Prints help information
* `-v` `--version` Prints version information

## Credit
Icon made by [Freepik](http://www.freepik.com/) from [Flaticon](https://www.flaticon.com/)