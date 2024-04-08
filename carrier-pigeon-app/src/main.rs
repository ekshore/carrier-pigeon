#[allow(unused_imports)]
use log::{debug, error, info, warn};

use iced::widget::{self, column, container};
use iced::Application;
use iced::{Command, Element};

#[allow(unused_imports)]
use reqwest::header::HeaderMap;
use simplelog::{ColorChoice, CombinedLogger, LevelFilter, TermLogger, TerminalMode};

use carrier_pigeon_lib::Collection;
#[allow(unused_imports)]
use carrier_pigeon_lib::{Header, PigeonError, Request};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Clone)]
enum Message {
    Loading,
    SelectCollection,
    ViewCollection(Option<String>),
}

#[derive(Debug, Default)]
enum App {
    #[default]
    Loading,
    SelectCollection(Option<Vec<String>>),
    ViewCollection(Option<String>),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (App::default(), Command::none())
    }

    fn title(&self) -> String {
        "Carrier Pigeon".into()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::Loading => *self = Self::Loading,
            Message::SelectCollection => {
                *self =
                    Self::SelectCollection(Some(vec!["Does".into(), "This".into(), "Work".into()]))
            }
            Message::ViewCollection(c) => *self = Self::ViewCollection(c),
        }
        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        match self {
            Self::Loading => test_component("Loading".into()),
            Self::SelectCollection(Some(collections)) => {
                let requests = request_list(collections);
                let load_btn = widget::button("Load").on_press(Message::Loading);
                let requests = requests.push(load_btn);
                container(requests).into()
            },
            Self::SelectCollection(None) => test_component("SelectCollection".into()),
            Self::ViewCollection(_collection) => test_component(_collection.as_ref().unwrap().into()),
        }
    }

    fn theme(&self) -> Self::Theme {
        Self::Theme::GruvboxDark
    }
}

fn request_list(
    request_names: &[String],
) -> widget::Column<'_, Message, iced::Theme, iced::Renderer> {
    let request_buttons: Vec<Element<Message, iced::Theme, iced::Renderer>> = request_names
        .iter()
        .map(|name| {
            let content = column![
                widget::text(name),
                widget::Rule::horizontal(5),
            ];
            widget::Button::new(content)
                .width(100)
                .on_press(Message::ViewCollection(Some(name.clone()))).into()
        })
        .collect();
    column(request_buttons)
}

fn test_component(display_text: String) -> Element<'static, Message, iced::Theme, iced::Renderer> {
    container(column![
        widget::text(display_text),
        widget::button("Loading").on_press(Message::Loading),
        widget::button("Select Collection").on_press(Message::SelectCollection),
        widget::button("View Collection").on_press(Message::ViewCollection(None)),
    ])
    .into()
}

#[tokio::main]
async fn main() -> Result<(), PigeonError> {
    let config = simplelog::ConfigBuilder::new()
        .add_filter_ignore_str("wgpu")
        .add_filter_ignore_str("iced_wgpu")
        .add_filter_ignore_str("naga")
        .build();
    let _logger = CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )]);

    App::run(iced::Settings::default())?;

    Ok(())
}
