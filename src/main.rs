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
    ScrollEntries(iced_widget::scrollable::Viewport),
}

const ENTRIES_SCROLL_CONTAINER_ID: &str = "entries_scroll_container";
const SEARCH_INPUT_ID: &str = "search_input";

struct State {
    query: String,
    selection_index: usize,
    entries: Vec<DesktopEntry>,
    filtered_entries: Vec<DesktopEntry>,
    entries_scroll_viewport: Option<iced_widget::scrollable::Viewport>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            selection_index: 0,
            query: String::new(),
            entries: Vec::new(),
            filtered_entries: Vec::new(),
            entries_scroll_viewport: None,
        }
    }
}

const ENTRY_HEIGHT: f32 = 48.0;

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
    fn selection_set(&mut self, index: usize) -> Task<Message> {
        self.selection_index = index;
        if self.selection_index >= self.filtered_entries.len() {
            self.selection_index = self.filtered_entries.len().saturating_sub(1);
        }
        let len = self.filtered_entries.len();

        let rel_y = if let Some(viewport) = &self.entries_scroll_viewport {
            let entry_height = ENTRY_HEIGHT;
            let entry_top = self.selection_index as f32 * entry_height;
            let entry_bottom = entry_top + entry_height;

            let viewport_top = viewport.absolute_offset().y;
            let viewport_height = viewport.bounds().height;
            let viewport_bottom = viewport_top + viewport_height;
            let is_fully_within_viewport = entry_top >= viewport_top && entry_bottom <= viewport_bottom;

            if is_fully_within_viewport {
                None
            } else {
                let total_height = (len as f32) * entry_height;
                let max_scroll = (total_height - viewport_height).max(0.0);

                let mut desired_offset_y = viewport_top;
                if entry_top < viewport_top {
                    desired_offset_y = entry_top;
                } else if entry_bottom > (viewport_top + viewport_height) {
                    desired_offset_y = entry_bottom - viewport_height;
                }

                let desired_offset_y = desired_offset_y.clamp(0.0, max_scroll);
                let rel_y = if max_scroll > 0.0 { desired_offset_y / max_scroll } else { 0.0 };

                Some(rel_y)
            }
        } else if len > 1 {
            Some(self.selection_index as f32 / (len.saturating_sub(1) as f32))
        } else {
            Some(0.0)
        };

        if let Some(rel_y) = rel_y {
            iced::widget::operation::snap_to(
                ENTRIES_SCROLL_CONTAINER_ID,
                iced::widget::scrollable::RelativeOffset { x: 0.0, y: rel_y },
            )
        } else {
            Task::none()
        }
    }
    fn selection_move(&mut self, direction: isize) -> Task<Message> {
        if direction < 0 {
            return self.selection_set(self.selection_index.saturating_sub(direction.abs() as usize));
        } else {
            return self.selection_set(self.selection_index.saturating_add(direction as usize));
        }
    }
}



fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::ScrollEntries(viewport) => {
            state.entries_scroll_viewport = Some(viewport);
        },
        Message::SetQuery(s) => {
            state.query = s;
            state.selection_index = 0;
            state.update_filtered_entries();
            return iced::widget::operation::snap_to(
                ENTRIES_SCROLL_CONTAINER_ID,
                iced::widget::scrollable::RelativeOffset::START,
            );
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
            return iced::widget::operation::snap_to(
                ENTRIES_SCROLL_CONTAINER_ID,
                iced::widget::scrollable::RelativeOffset::START,
            );
        },
        Message::Event(e) => {
            match e {
                iced_core::event::Event::Window(iced_core::window::Event::Unfocused) => {
                    return iced::exit();
                },
                iced_core::event::Event::Window(iced_core::window::Event::Focused) => {
                    return iced::widget::operation::focus(SEARCH_INPUT_ID);
                },
                iced::Event::InputMethod(iced_core::input_method::Event::Closed) => {
                    return iced::widget::operation::focus(SEARCH_INPUT_ID);
                },
                iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                        return iced::exit();
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) {
                        return state.selection_move(-1);
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) {
                        return state.selection_move(1);
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp) {
                        return state.selection_set(0);
                    }
                    if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown) {
                        return state.selection_set(state.filtered_entries.len().saturating_sub(1));
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
    }
    
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
        .height(iced::Length::Fixed(ENTRY_HEIGHT))
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
            .id(SEARCH_INPUT_ID)
            .style(|theme, _status| {
                let mut s = iced::widget::text_input::default(&theme, _status);
                s.border.color = iced::Color::from_rgba(0.0, 0.0, 0.0, 0.0);
                s.border.width = 0.0;
                s
            })
            .padding(10),
        iced::widget::scrollable(iced::widget::column(elements))
            .width(iced::Length::Fill)
            .on_scroll(|v|Message::ScrollEntries(v))
            .id(ENTRIES_SCROLL_CONTAINER_ID)
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
