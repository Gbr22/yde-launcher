use std::{path::PathBuf, thread};

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Element, Task};

use crate::{data::APP_DATA, entry::Entry};

mod data;
mod entry;

#[derive(Debug, Clone)]
enum Message {
    Event(iced::Event),
    SetQuery(String),
    PressEntry(String)
}

struct State {
    query: String,
    selection_index: usize,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selection_index: 0,
            query: String::new()
        }
    }
}

fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::SetQuery(s) => {
            state.query = s;
            state.selection_index = 0;
        },
        Message::Event(iced::Event::Window(iced::window::Event::CloseRequested)) => {
            return iced::exit();
        },
        Message::PressEntry(id) =>{
            let lock = APP_DATA.read().unwrap();
            let entry = lock.entries.iter().find(|e| e.id() == id);
            if let Some(entry) = entry {
                println!("Entry: {:?}", entry);
            }
        },
        Message::Event(e) => {
            match e {
                iced_core::event::Event::Window(iced_core::window::Event::Unfocused) => {
                    return iced::exit();
                },
                iced_core::event::Event::Window(iced_core::window::Event::Focused) => {
                    return iced::widget::operation::focus("search_input");
                },
                iced::Event::InputMethod(iced_core::input_method::Event::Closed) => {
                    return iced::widget::operation::focus("search_input");
                },
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                        return iced::exit();
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) {
                        state.selection_index = state.selection_index.saturating_sub(1);
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) {
                        state.selection_index = state.selection_index.saturating_add(1);
                    }
                },
                iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }) => {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                        return iced::exit();
                    }
                },
                _ => {}
            };
        },
        _ => {}
    };
    
    Task::none()
}

fn view(state: &State) -> Element<Message> {
    let lock = APP_DATA.read().unwrap();
    
    let elements: Vec<Element<Message>> = lock.entries.iter().enumerate().flat_map(|(i, entry)|{
        let is_selected = i == state.selection_index;
        let does_match = if state.query.is_empty() {
            true
        } else {
            let matcher = SkimMatcherV2::default();
            matcher.fuzzy_match(entry.title(), &state.query).is_some()
        };
        if !does_match {
            return None;
        }
        let mut content = iced::widget::Column::new();
        content = content.push(
            iced::widget::text(entry.title().to_string())
                .size(15)
        );
        if let Some(descrption) = entry.description() {
            content = content.push(
                iced::widget::text(descrption.to_string())
                    .style(|theme: &iced::Theme|{
                        let mut s = iced::widget::text::Style::default();
                        s.color = Some(theme.palette().text.scale_alpha(0.6));
                        s
                    })
                    .size(13)
            );
        }

        let image_elem: iced::Element<Message> = if let Some(icon_path) = entry.icon_path() {
            iced::widget::Image::new(icon_path.clone())
                .width(iced::Length::Fixed(40.0))
                .height(iced::Length::Fixed(40.0))
                .into()
        } else {
            iced::widget::space()
                .width(iced::Length::Fixed(40.0))
                .height(iced::Length::Fixed(40.0))
                .into()
        };

        Some(iced::widget::button(
            iced::widget::row![
                image_elem,
                iced::widget::space().width(iced::Length::Fixed(10.0)),
                content
            ]
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Fixed(48.0))
        .style(move |theme: &iced::Theme, status| {
            let mut s = iced::widget::button::Style::default();
            s.text_color = theme.palette().text;

            if is_selected {
                s.text_color = theme.palette().primary;
                s.background = Some(iced::Background::Color(theme.palette().primary.scale_alpha(0.1)));
            }

            s
        })
        .on_press(Message::PressEntry(entry.id().to_string()))
        .into())
    }).collect();

    iced::widget::column![
        iced::widget::text_input("Search", &state.query)
            .on_input(|s| Message::SetQuery(s))
            .id("search_input")
            .style(|theme, status| {
                let mut s = iced::widget::text_input::default(&theme, status);
                s.border.color = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0);
                s.border.width = 0.0;
                s
            })
            .padding(10),
        iced::widget::scrollable(iced::widget::column(elements))
            .width(iced::Length::Fill)
    ].into()
}

pub fn main() -> iced::Result {
    thread::spawn(||{
        data::load_app_data();
    });

    iced::application(||{
        State::default()
    }, update, view)
        .subscription(|_state| iced::event::listen().map(Message::Event))
        .window(iced::window::Settings {
            decorations: false,
            transparent: true,
            resizable: false,
            size: iced::Size { width: 500.0, height: 400.0 },
            level: iced::window::Level::AlwaysOnTop,
            ..Default::default()
        })
        .run()
}
