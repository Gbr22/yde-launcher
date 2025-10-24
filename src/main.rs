use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use iced::{Element, Task};
use crate::entry::Entry;
use crate::data::DesktopEntry;

mod data;
mod entry;

#[derive(Debug, Clone)]
enum Message {
    Event(iced::Event),
    SetQuery(String),
    PressEntry(String),
    LoadEntries(Vec<DesktopEntry>),
}

struct State {
    query: String,
    selection_index: usize,
    entries: Vec<DesktopEntry>,
    filtered_entries: Vec<DesktopEntry>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selection_index: 0,
            query: String::new(),
            entries: Vec::new(),
            filtered_entries: Vec::new(),
        }
    }
}

impl State {
    fn update_filtered_entries(&mut self) {
        let matcher = SkimMatcherV2::default();
        let mut vec = self.entries.iter().flat_map(|entry| {
            if self.query.is_empty() {
                Some((0, entry))
            } else {
                matcher.fuzzy_match(entry.title(), &self.query).map(|score|{
                    (score, entry)
                })
            }
        }).collect::<Vec<_>>();
        vec.sort_by(|a,b|a.1.title().cmp(b.1.title()));
        vec.sort_by(|a, b| b.0.cmp(&a.0));
        self.filtered_entries = vec.iter().map(|(_, entry)| (*entry).clone()).collect();
    }
}

impl State {
    fn selection_move(&mut self, direction: isize) {
        if direction < 0 {
            self.selection_index = self.selection_index.saturating_sub(1);
        } else if direction > 0 {
            self.selection_index = self.selection_index.saturating_add(1);
            if self.selection_index >= self.filtered_entries.len() {
                self.selection_index = self.filtered_entries.len().saturating_sub(1);
            }
        }
    }
}

fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::SetQuery(s) => {
            state.query = s;
            state.selection_index = 0;
            state.update_filtered_entries();
        },
        Message::Event(iced::Event::Window(iced::window::Event::CloseRequested)) => {
            return iced::exit();
        },
        Message::PressEntry(id) =>{
            let entry = state.entries.iter().find(|e| e.id() == id);
            if let Some(entry) = entry {
                println!("Entry: {:?}", entry);
            }
        },
        Message::LoadEntries(entries) => {
            state.entries = entries;
            state.selection_index = 0;
            state.update_filtered_entries();
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
                        state.selection_move(-1);
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) {
                        state.selection_move(1);
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp) {
                        state.selection_index = 0;
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown) {
                        state.selection_index = state.filtered_entries.len().saturating_sub(1);
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

fn view(state: &State) -> Element<'_, Message> {
    let elements: Vec<Element<Message>> = state.filtered_entries.iter().enumerate().flat_map(|(i, entry)|{
        let is_selected = i == state.selection_index;
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
    .style(move |theme: &iced::Theme, _status| {
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
            .style(|theme, _status| {
                let mut s = iced::widget::text_input::default(&theme, _status);
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
    iced::application(|| {
        (
            State::default(),
            Task::perform(async move {
                data::get_desktop_entries()
            }, |entries| Message::LoadEntries(entries)),
        )
    }, update, view)
        .transparent(true)
        .antialiasing(true)
        .centered()
        .window_size((500.0, 400.0))
        .level(iced::window::Level::AlwaysOnTop)
        .decorations(false)
        .subscription(|_state| iced::event::listen().map(Message::Event))
        .run()
}
