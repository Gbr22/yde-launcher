use iced::{Element, Task};

#[derive(Debug, Clone)]
enum Message {
    Event(iced::Event),
    SetQuery(String),
}

struct State {
    query: String,
}

impl Default for State {
    fn default() -> Self {
        Self {
            query: String::new()
        }
    }
}

fn update(state: &mut State, message: Message) -> iced::Task<Message> {
    match message {
        Message::SetQuery(s) => state.query = s,
        Message::Event(iced::Event::Window(iced::window::Event::CloseRequested)) => {
            return iced::exit();
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
    ].into()
}

pub fn main() -> iced::Result {
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
