use iced::widget::{button, text};
use iced::{Element};

#[derive(Debug, Clone)]
enum Message {
    Increment,
}

fn update(counter: &mut u64, message: Message) {
    match message {
        Message::Increment => *counter += 1,
    }
}

fn view(counter: &u64) -> Element<Message> {
    button(text(counter)).on_press(Message::Increment).into()
}

pub fn main() -> iced::Result {
    iced::run("yde-launcher", update, view)
}
