use app_bundler::*;
use iced::{button, Sandbox, Element, Column, Text, Button, window, settings, Image, Length, image, Row, text_input, TextInput};
use nfd::*;
use app_bundler::ExitCode::{FileDialogError, Success, WrongFileFormat, BinaryNotFound};
use std::process;
use std::path::PathBuf;
use rust_embed::*;

const IMG_BINARY: &str = "executable.png";
const IMG_ICON: &str = "icon.png";

const TITLE: &str = "AppBundler";
const WIDTH: u32 = 320;
const HEIGHT: u32 = 208;
const BTN_DIM: Length = Length::Units(50);
const BTN_PADDING: u16 = 3;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

#[derive(Default)]
struct GuiState {
    btn_binary: button::State,
    btn_icon: button::State,
    btn_bundle: button::State,
    txt_name: text_input::State,

    app_name: String,

    data: DataParsed,
    feedback: ExitCode,
    assets: GuiAssets,
}

struct GuiAssets {
    img_binary: image::Handle,
    img_icon: image::Handle,
}

#[derive(Debug, Clone)]
pub enum Message {
    BtnBinaryPressed,
    BtnIconPressed,
    BtnBundlePressed,
    TxtNameChanged(String),
}

impl GuiAssets {
    pub fn new() -> Self {
        let binary_raw = Assets::get(IMG_BINARY).unwrap();
        let icon_raw = Assets::get(IMG_ICON).unwrap();



        GuiAssets {
            img_binary: image::Handle::from_memory(binary_raw.to_vec()),
            img_icon: image::Handle::from_memory(icon_raw.to_vec()),
        }
    }
}

impl Default for GuiAssets {
    fn default() -> Self {
        GuiAssets::new()
    }
}

impl Sandbox for GuiState {
    type Message = Message;

    fn new() -> Self {
        GuiState::default()
    }

    fn title(&self) -> String {
        String::from(TITLE)
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::BtnBinaryPressed => {
                let ret = nfd::open_file_dialog(None, None).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    process::exit(FileDialogError as i32)
                });

                match ret {
                    Response::Cancel => {}
                    Response::OkayMultiple(_) => {}
                    Response::Okay(path) => { self.data.binary = PathBuf::from(path) }
                }
            }
            Message::BtnBundlePressed => {
                if self.data.binary.is_file() {
                    self.data.name = Some(PathBuf::from(&self.app_name));

                    self.feedback = match bundle(&self.data) {
                        Ok(()) => Success,
                        Err(err) => err,
                    }
                } else {
                    self.feedback = BinaryNotFound
                }
            }
            Message::BtnIconPressed => {
                let ret = nfd::open_file_dialog(Some("icns"), None).unwrap_or_else(|e| {
                    eprintln!("{}", e);
                    process::exit(FileDialogError as i32)
                });

                match ret {
                    Response::Cancel => {}
                    Response::OkayMultiple(_) => {}
                    Response::Okay(val) => {
                        let path = PathBuf::from(val);
                        if is_icns(&path) {
                            self.data.icon = Some(path)
                        } else {
                            self.feedback = WrongFileFormat
                        }
                    }
                }
            }
            Message::TxtNameChanged(val) => {
                self.app_name = String::from(val.trim())
            }
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let binary_img = Image::new(self.assets.img_binary.clone())
            .width(BTN_DIM)
            .height(BTN_DIM);
        let icon_img = Image::new(self.assets.img_icon.clone())
            .width(BTN_DIM)
            .height(BTN_DIM);

        let btn_binary = Button::new(&mut self.btn_binary, binary_img)
            .on_press(Message::BtnBinaryPressed)
            .padding(BTN_PADDING);

        let btn_icon = Button::new(&mut self.btn_icon, icon_img)
            .on_press(Message::BtnIconPressed)
            .padding(BTN_PADDING);

        let btn_bundle = Button::new(&mut self.btn_bundle, Text::new("Bundle"))
            .on_press(Message::BtnBundlePressed);

        let txt_name = TextInput::new(&mut self.txt_name, "App Name", self.app_name.as_str(), Message::TxtNameChanged);

        let label_bin = Text::new("Binary");
        let label_icon = Text::new("Icon");

        let feedback = Text::new(format!("Feedback: {:?}", self.feedback));

        let binary_col = Column::new()
            .padding(10)
            .push(label_bin)
            .push(btn_binary);

        let icon_col = Column::new()
            .padding(10)
            .push(label_icon)
            .push(btn_icon);

        let btn_row = Row::new()
            .padding(10)
            .push(binary_col)
            .push(icon_col);

        Column::new()
            .padding(10)
            .push(btn_row)
            .push(
                Row::new()
                    .push(txt_name)
                    .push(btn_bundle)
            )
            .push(feedback)
            .into()
    }
}

fn main() {
    let settings = settings::Settings {
        window: window::Settings {
            size: (WIDTH, HEIGHT),
            min_size: None,
            max_size: None,
            resizable: false,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None
        },
        flags: (),
        default_font: None,
        default_text_size: 20,
        antialiasing: false
    };

    let _ = GuiState::run(settings);
}
