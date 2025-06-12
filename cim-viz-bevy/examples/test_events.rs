//! Simple test to verify event logging is working

use bevy::prelude::*;
use cim_viz_bevy::*;

fn main() {
    println!("Starting event test demo...");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(CimVizPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, test_events)
        .run();
}

fn setup(
    bridge: Res<CategoricalBridge>,
) {
    println!("Setup: Sending test event");

    let event = DomainEvent::NodeAdded {
        graph_id: Default::default(),
        node_id: Default::default(),
        position: Some(Vec3::ZERO),
        metadata: serde_json::Value::Null,
    };

    bridge.send_domain_event(event);
}

fn test_events(
    mut events: EventReader<DomainEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    bridge: Res<CategoricalBridge>,
) {
    for event in events.read() {
        println!("Received event: {:?}", event);
    }

    if keyboard.just_pressed(KeyCode::Space) {
        println!("SPACE pressed - sending test event");

        let event = DomainEvent::NodeAdded {
            graph_id: Default::default(),
            node_id: Default::default(),
            position: Some(Vec3::new(1.0, 2.0, 3.0)),
            metadata: serde_json::json!({"test": true}),
        };

        bridge.send_domain_event(event);
    }
}
