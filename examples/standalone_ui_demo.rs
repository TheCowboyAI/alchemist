//! Completely standalone UI demo - no alchemist library dependency

use iced::{Element, Task, window, Length, Theme};
use iced::widget::{button, column, text, container, row, Space};

fn main() -> iced::Result {
    println!("🚀 Starting Alchemist UI Demo...");
    println!("This proves the UI framework is working!");
    
    iced::application("Alchemist UI Demo", update, view)
        .window(window::Settings {
            size: iced::Size::new(900.0, 700.0),
            position: window::Position::Centered,
            ..Default::default()
        })
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Default)]
struct State {
    counter: i32,
    nats_connected: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
    ToggleNats,
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => {
            state.counter += 1;
            println!("✅ Counter incremented to: {}", state.counter);
        }
        Message::Decrement => {
            state.counter -= 1;
            println!("✅ Counter decremented to: {}", state.counter);
        }
        Message::ToggleNats => {
            state.nats_connected = !state.nats_connected;
            println!("✅ NATS connection toggled to: {}", state.nats_connected);
        }
    }
    Task::none()
}

fn view(state: &State) -> Element<'_, Message> {
    let header = container(
        column![
            text("🧪 Alchemist UI Working!").size(36),
            text("This is a minimal demo proving the UI framework functions correctly").size(16),
        ]
        .spacing(10)
    )
    .padding(20)
    .style(container::rounded_box);

    let status = container(
        column![
            text("System Status").size(24),
            row![
                text("NATS Status: "),
                button(if state.nats_connected { "Connected ✓" } else { "Disconnected ✗" })
                    .on_press(Message::ToggleNats)
                    .style(if state.nats_connected { button::success } else { button::danger }),
            ].spacing(10),
            text(format!("Counter Value: {}", state.counter)).size(18),
        ]
        .spacing(15)
    )
    .padding(20)
    .style(container::rounded_box);

    let controls = container(
        row![
            button("➕ Increment")
                .on_press(Message::Increment)
                .style(button::primary),
            button("➖ Decrement")
                .on_press(Message::Decrement)
                .style(button::secondary),
        ]
        .spacing(20)
    )
    .padding(20);

    let info = container(
        column![
            text("UI Features Demonstrated:").size(20),
            text("• Iced framework is properly installed and working"),
            text("• Buttons respond to clicks"),
            text("• State management functions correctly"),
            text("• Theme and styling applied"),
            text("• Window rendering without errors"),
        ]
        .spacing(5)
    )
    .padding(20)
    .style(container::rounded_box);

    let content = column![
        header,
        status,
        controls,
        info,
    ]
    .spacing(20);

    container(content)
        .padding(30)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}