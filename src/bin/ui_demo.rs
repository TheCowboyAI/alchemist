//! Minimal UI demo that compiles and runs

use iced::{Element, Task, window, Length};
use iced::widget::{button, column, text, container};

fn main() -> iced::Result {
    println!("Starting Alchemist UI Demo...");
    iced::application("Alchemist UI Demo", update, view)
        .window(window::Settings {
            size: iced::Size::new(800.0, 600.0),
            position: window::Position::Centered,
            ..Default::default()
        })
        .run()
}

#[derive(Default)]
struct State {
    counter: i32,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => {
            state.counter += 1;
            println!("Counter: {}", state.counter);
        }
        Message::Decrement => {
            state.counter -= 1;
            println!("Counter: {}", state.counter);
        }
    }
    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    let content = column![
        text("Alchemist UI is Working!").size(32),
        text(format!("Counter: {}", state.counter)).size(24),
        button("Increment").on_press(Message::Increment),
        button("Decrement").on_press(Message::Decrement),
    ]
    .spacing(20);

    container(content)
        .padding(40)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}