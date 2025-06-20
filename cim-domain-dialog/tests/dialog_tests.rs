//! Tests for the Dialog domain

use cim_domain_dialog::{
    Dialog, DialogType,
    Participant, ParticipantType, ParticipantRole,
    Turn, TurnType, Message, MessageIntent,
    Topic, ContextVariable, ContextScope,
};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_create_dialog() {
    // Create a user participant
    let user = Participant {
        id: Uuid::new_v4(),
        participant_type: ParticipantType::Human,
        role: ParticipantRole::Primary,
        name: "Test User".to_string(),
        metadata: HashMap::new(),
    };

    // Create a dialog
    let dialog = Dialog::new(
        Uuid::new_v4(),
        DialogType::Direct,
        user.clone(),
    );

    assert_eq!(dialog.dialog_type(), DialogType::Direct);
    assert_eq!(dialog.participants().len(), 1);
    assert!(dialog.participants().contains_key(&user.id));
}

#[test]
fn test_add_participant() {
    // Create initial dialog
    let user = Participant {
        id: Uuid::new_v4(),
        participant_type: ParticipantType::Human,
        role: ParticipantRole::Primary,
        name: "Test User".to_string(),
        metadata: HashMap::new(),
    };

    let mut dialog = Dialog::new(
        Uuid::new_v4(),
        DialogType::Direct,
        user,
    );

    // Add an AI agent participant
    let agent = Participant {
        id: Uuid::new_v4(),
        participant_type: ParticipantType::AIAgent,
        role: ParticipantRole::Assistant,
        name: "AI Assistant".to_string(),
        metadata: HashMap::new(),
    };

    let events = dialog.add_participant(agent.clone()).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(dialog.participants().len(), 2);
    assert!(dialog.participants().contains_key(&agent.id));
}

#[test]
fn test_add_turn() {
    // Create dialog with participant
    let user_id = Uuid::new_v4();
    let user = Participant {
        id: user_id,
        participant_type: ParticipantType::Human,
        role: ParticipantRole::Primary,
        name: "Test User".to_string(),
        metadata: HashMap::new(),
    };

    let mut dialog = Dialog::new(
        Uuid::new_v4(),
        DialogType::Direct,
        user,
    );

    // Add a turn
    let turn = Turn::new(
        1,
        user_id,
        Message::text("Hello, world!").with_intent(MessageIntent::Statement),
        TurnType::UserQuery,
    );

    let events = dialog.add_turn(turn).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(dialog.turns().len(), 1);
}

#[test]
fn test_context_switching() {
    // Create dialog
    let user = Participant {
        id: Uuid::new_v4(),
        participant_type: ParticipantType::Human,
        role: ParticipantRole::Primary,
        name: "Test User".to_string(),
        metadata: HashMap::new(),
    };

    let mut dialog = Dialog::new(
        Uuid::new_v4(),
        DialogType::Direct,
        user,
    );

    // Switch to a topic
    let topic = Topic::new(
        "Weather Discussion",
        vec!["weather".to_string(), "temperature".to_string()],
    );

    let events = dialog.switch_topic(topic).unwrap();
    assert_eq!(events.len(), 1);
    assert!(dialog.current_topic().is_some());
    assert_eq!(dialog.current_topic().unwrap().name, "Weather Discussion");
}

#[test]
fn test_dialog_lifecycle() {
    // Create and pause dialog
    let user = Participant {
        id: Uuid::new_v4(),
        participant_type: ParticipantType::Human,
        role: ParticipantRole::Primary,
        name: "Test User".to_string(),
        metadata: HashMap::new(),
    };

    let mut dialog = Dialog::new(
        Uuid::new_v4(),
        DialogType::Direct,
        user,
    );

    // Pause the dialog
    let pause_events = dialog.pause().unwrap();
    assert_eq!(pause_events.len(), 1);
    assert_eq!(dialog.status(), cim_domain_dialog::DialogStatus::Paused);

    // Resume the dialog
    let resume_events = dialog.resume().unwrap();
    assert_eq!(resume_events.len(), 1);
    assert_eq!(dialog.status(), cim_domain_dialog::DialogStatus::Active);

    // End the dialog
    let end_events = dialog.end(Some("Test completed".to_string())).unwrap();
    assert_eq!(end_events.len(), 1);
    assert_eq!(dialog.status(), cim_domain_dialog::DialogStatus::Ended);
}

#[test]
fn test_context_variables() {
    // Create dialog
    let user = Participant {
        id: Uuid::new_v4(),
        participant_type: ParticipantType::Human,
        role: ParticipantRole::Primary,
        name: "Test User".to_string(),
        metadata: HashMap::new(),
    };

    let mut dialog = Dialog::new(
        Uuid::new_v4(),
        DialogType::Direct,
        user,
    );

    // Add a context variable
    let variable = ContextVariable {
        name: "user_preference".to_string(),
        value: serde_json::json!("dark_mode"),
        scope: ContextScope::Dialog,
        set_at: Utc::now(),
        expires_at: None,
        source: dialog.id(),
    };

    let events = dialog.add_context_variable(variable).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(dialog.context().variables.len(), 1);
    assert!(dialog.context().variables.contains_key("user_preference"));
} 