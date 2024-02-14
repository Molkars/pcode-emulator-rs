use iced::{Application, Command, Element, executor, Font, Renderer, Settings, Theme};
use iced::widget::Text;
use pcode::binary::Binary;

pub(crate) fn run(binary: Binary) {
    Debugger::run(Settings {
        id: None,
        window: Default::default(),
        flags: binary,
        default_font: Font::DEFAULT,
        default_text_size: 11.0,
        antialiasing: true,
        exit_on_close_request: true,
    }).unwrap()
}

pub struct Debugger {
    binary: Binary,
}

#[derive(Debug, Clone)]
pub enum Message {}

impl Application for Debugger {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = Binary;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                binary: flags,
            },
            Command::none()
        )
    }

    fn title(&self) -> String {
        "Emulator".to_string()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {}
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        Text::new("Hello, world!").into()
    }
}
