use iced::widget::text_input::Id;
use iced::widget::{button, text};
use iced::{Element, Task};

#[derive(Debug, Clone)]
enum Message {
    Event(iced::Event),
    Increment,
    SetQuery(String),
    Focus(Id)
}

fn exit() {
    std::process::exit(0);
}

struct State {
    counter: u64,
    query: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            counter: 0,
            query: String::new()
        }
    }
}

pub fn search_id() -> iced::widget::text_input::Id {
    return iced::widget::text_input::Id::new("search_input");
}

fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::Increment => state.counter += 1,
        Message::SetQuery(s) => state.query = s,
        Message::Event(iced::Event::Window(iced::window::Event::CloseRequested)) => {
            exit();
        },
        Message::Event(iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. })) => {
            if key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) {
                exit();
            }
        },
        Message::Focus(id) => {
            return iced::widget::text_input::focus(id);
        },
        _ => {}
    };
    
    Task::none()
}

fn view(state: &State) -> Element<Message> {
    iced::widget::column![
        button(text(state.counter)).on_press(Message::Increment),
        iced::widget::text_input("Search", &state.query)
            .on_input(|s| Message::SetQuery(s))
            .id(search_id()),
    ].into()
}

pub fn main() -> iced::Result {
    iced::application("yde-launcher", update, view)
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
