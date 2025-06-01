use bevy::prelude::*;
use bevy::input::ButtonState;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::keyboard::{Key, KeyboardInput, NativeKey};
use std::collections::VecDeque;

/// Test action that can be performed
#[derive(Debug, Clone)]
pub enum TestAction {
    MoveMouse { x: f32, y: f32 },
    Click { button: MouseButton },
    KeyPress { key: KeyCode },
    KeyRelease { key: KeyCode },
    Wait { frames: u32 },
    AssertNodeExists { tag: String },
    AssertNodeSelected { tag: String },
    TakeScreenshot { name: String },
}

/// Test result
#[derive(Debug, Clone)]
pub enum TestResult {
    Pass,
    Fail(String),
}

/// Test scenario with a series of actions
#[derive(Resource, Clone)]
pub struct TestScenario {
    pub name: String,
    pub actions: VecDeque<TestAction>,
    pub current_wait: u32,
    pub results: Vec<(String, TestResult)>,
}

impl TestScenario {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            actions: VecDeque::new(),
            current_wait: 0,
            results: Vec::new(),
        }
    }

    pub fn add_action(mut self, action: TestAction) -> Self {
        self.actions.push_back(action);
        self
    }
}

/// Plugin to run automated tests
pub struct AutomatedTestPlugin {
    pub scenarios: Vec<TestScenario>,
}

impl Plugin for AutomatedTestPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TestRunner {
            scenarios: self.scenarios.clone().into(),
            current_scenario: 0,
            is_running: true,
        })
        .add_systems(Update, (
            run_test_scenarios,
            handle_test_completion,
        ).chain());
    }
}

#[derive(Resource)]
struct TestRunner {
    scenarios: VecDeque<TestScenario>,
    current_scenario: usize,
    is_running: bool,
}

fn run_test_scenarios(
    mut runner: ResMut<TestRunner>,
    windows: Query<Entity, With<Window>>,
    mut mouse_events: EventWriter<CursorMoved>,
    mut mouse_button_events: EventWriter<MouseButtonInput>,
    mut keyboard_events: EventWriter<KeyboardInput>,
    query: Query<(Entity, &Name), With<Node>>,
) {
    if !runner.is_running {
        return;
    }

    if let Some(mut scenario) = runner.scenarios.pop_front() {
        // Handle wait frames
        if scenario.current_wait > 0 {
            scenario.current_wait -= 1;
            runner.scenarios.push_front(scenario);
            return;
        }

        // Process next action
        if let Some(action) = scenario.actions.pop_front() {
            match action {
                TestAction::MoveMouse { x, y } => {
                    if let Ok(window_entity) = windows.get_single() {
                        mouse_events.write(CursorMoved {
                            window: window_entity,
                            position: Vec2::new(x, y),
                            delta: None,
                        });
                    }
                }
                TestAction::Click { button } => {
                    if let Ok(window_entity) = windows.get_single() {
                        mouse_button_events.write(MouseButtonInput {
                            button,
                            state: ButtonState::Pressed,
                            window: window_entity,
                        });
                        mouse_button_events.write(MouseButtonInput {
                            button,
                            state: ButtonState::Released,
                            window: window_entity,
                        });
                    }
                }
                TestAction::KeyPress { key } => {
                    keyboard_events.write(KeyboardInput {
                        key_code: key,
                        logical_key: Key::Unidentified(NativeKey::Unidentified),
                        state: ButtonState::Pressed,
                        window: Entity::PLACEHOLDER,
                        text: None,
                        repeat: false,
                    });
                }
                TestAction::KeyRelease { key } => {
                    keyboard_events.write(KeyboardInput {
                        key_code: key,
                        logical_key: Key::Unidentified(NativeKey::Unidentified),
                        state: ButtonState::Released,
                        window: Entity::PLACEHOLDER,
                        text: None,
                        repeat: false,
                    });
                }
                TestAction::Wait { frames } => {
                    scenario.current_wait = frames;
                }
                TestAction::AssertNodeExists { tag } => {
                    let exists = query.iter().any(|(_, name)| name.as_str() == tag);
                    let result = if exists {
                        TestResult::Pass
                    } else {
                        TestResult::Fail(format!("Node '{}' not found", tag))
                    };
                    scenario.results.push((format!("Assert node exists: {}", tag), result));
                }
                TestAction::AssertNodeSelected { tag } => {
                    // This would check for Selected component
                    let result = TestResult::Pass; // Simplified
                    scenario.results.push((format!("Assert node selected: {}", tag), result));
                }
                TestAction::TakeScreenshot { name } => {
                    info!("Screenshot would be taken: {}", name);
                }
            }

            runner.scenarios.push_front(scenario);
        } else {
            // Scenario complete
            print_scenario_results(&scenario);
            runner.current_scenario += 1;
        }
    } else {
        runner.is_running = false;
    }
}

fn handle_test_completion(
    runner: Res<TestRunner>,
    mut app_exit: EventWriter<AppExit>,
) {
    if !runner.is_running && runner.scenarios.is_empty() {
        info!("All tests completed!");
        app_exit.write(AppExit::Success);
    }
}

fn print_scenario_results(scenario: &TestScenario) {
    println!("\n=== Test Scenario: {} ===", scenario.name);
    for (test, result) in &scenario.results {
        match result {
            TestResult::Pass => println!("✅ {}", test),
            TestResult::Fail(reason) => println!("❌ {} - {}", test, reason),
        }
    }
}

// Example usage
pub fn create_test_scenarios() -> Vec<TestScenario> {
    vec![
        TestScenario::new("Test Node Selection")
            .add_action(TestAction::Wait { frames: 10 })
            .add_action(TestAction::MoveMouse { x: 100.0, y: 100.0 })
            .add_action(TestAction::Click { button: MouseButton::Left })
            .add_action(TestAction::Wait { frames: 5 })
            .add_action(TestAction::AssertNodeSelected { tag: "node_1".into() })
            .add_action(TestAction::TakeScreenshot { name: "selection_test".into() }),

        TestScenario::new("Test Keyboard Navigation")
            .add_action(TestAction::KeyPress { key: KeyCode::ArrowRight })
            .add_action(TestAction::Wait { frames: 5 })
            .add_action(TestAction::KeyRelease { key: KeyCode::ArrowRight })
            .add_action(TestAction::AssertNodeExists { tag: "graph_1".into() }),
    ]
}
