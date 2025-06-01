use enigo::{Button, Coordinate, Direction::Click, Enigo, Key, Keyboard, Mouse, Settings};
use std::thread;
use std::time::Duration;

/// Example of using Enigo for automated UI testing with Bevy
/// Note: This requires the app to be running with a visible window
pub fn automated_ui_test() -> Result<(), Box<dyn std::error::Error>> {
    // Give the app time to start
    thread::sleep(Duration::from_secs(2));

    let mut enigo = Enigo::new(&Settings::default())?;

    // Move to a specific button position
    enigo.move_mouse(500, 200, Coordinate::Abs)?;
    thread::sleep(Duration::from_millis(100));

    // Click the button
    enigo.button(Button::Left, Click)?;
    thread::sleep(Duration::from_millis(100));

    // Type some text
    enigo.text("Hello from automated test")?;

    // Use keyboard shortcuts
    enigo.key(Key::Control, Direction::Press)?;
    enigo.key(Key::Unicode('a'), Click)?;
    enigo.key(Key::Control, Direction::Release)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Child};

    struct AppHandle {
        child: Child,
    }

    impl Drop for AppHandle {
        fn drop(&mut self) {
            let _ = self.child.kill();
        }
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_full_app_interaction() {
        // Start your Bevy app
        let app = AppHandle {
            child: Command::new("cargo")
                .args(&["run", "--release"])
                .spawn()
                .expect("Failed to start app"),
        };

        // Run automated test
        automated_ui_test().expect("Test failed");

        // App will be killed when app goes out of scope
    }
}
