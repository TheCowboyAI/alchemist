//! Minimal UI test to verify Iced is working

fn main() {
    println!("Starting minimal UI test...");
    
    // Just create a simple window to test
    if let Err(e) = run_app() {
        eprintln!("Error running app: {}", e);
    }
}

fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    use iced::{Element, Task, window, Length};
    use iced::widget::{button, column, text, container};
    
    #[derive(Default)]
    struct MinimalApp;

    #[derive(Debug, Clone)]
    enum Message {
        Click,
    }

    impl MinimalApp {
        fn update(&mut self, message: Message) -> Task<Message> {
            match message {
                Message::Click => {
                    println!("Button clicked!");
                    Task::none()
                }
            }
        }

        fn view(&self) -> Element<Message> {
            let content = column![
                text("Minimal Iced Test").size(24),
                button("Click Me").on_press(Message::Click),
            ]
            .spacing(20);

            container(content)
                .padding(20)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        }
    }

    iced::application("Minimal Test", MinimalApp::update, MinimalApp::view)
        .window(window::Settings {
            size: iced::Size::new(400.0, 300.0),
            ..Default::default()
        })
        .run()
        .map_err(|e| e.into())
}