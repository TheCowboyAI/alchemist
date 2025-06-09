//! Minimal test to catch Bevy system parameter conflicts

use bevy::prelude::*;

#[test]
fn test_conflicting_event_access() {
    let mut app = App::new();

    // Add minimal plugins
    app.add_plugins(MinimalPlugins);

    // Add an event type
    app.add_event::<TestEvent>();

    // Add two systems that would conflict
    app.add_systems(Update, (
        system_that_reads_events,
        system_that_writes_events,
    ));

    // This should panic if there's a conflict
    app.update();
}

#[derive(Event)]
struct TestEvent;

fn system_that_reads_events(mut reader: EventReader<TestEvent>) {
    for _ in reader.read() {
        // Read events
    }
}

fn system_that_writes_events(mut writer: EventWriter<TestEvent>) {
    writer.write(TestEvent);
}

#[test]
#[should_panic(expected = "conflicts with a previous")]
fn test_res_resmut_conflict() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);

    // Add a resource
    app.insert_resource(TestResource(0));

    // Add systems that conflict on resource access
    app.add_systems(Update, (
        system_that_reads_resource,
        system_that_mutates_resource,
    ));

    // This SHOULD panic with a conflict error
    app.update();
}

#[derive(Resource)]
struct TestResource(i32);

fn system_that_reads_resource(resource: Res<TestResource>) {
    let _ = resource.0;
}

fn system_that_mutates_resource(mut resource: ResMut<TestResource>) {
    resource.0 += 1;
}

#[test]
fn test_chained_systems_no_conflict() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_event::<TestEvent>();

    // When chained, systems run sequentially so no conflict
    app.add_systems(Update, (
        system_that_writes_events,
        system_that_reads_events,
    ).chain());

    // This should NOT panic
    app.update();
}

#[test]
#[should_panic(expected = "ResMut<bevy_ecs::event::collections::Events")]
fn test_event_reader_writer_internal_conflict() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_event::<TestEvent>();

    // This simulates what's happening in our code:
    // EventReader internally uses Res<Events<T>>
    // EventWriter internally uses ResMut<Events<T>>
    // Running them in parallel causes a conflict
    app.add_systems(Update, (
        |mut reader: EventReader<TestEvent>, mut writer: EventWriter<TestEvent>| {
            // This system has both reader and writer - no conflict within same system
            for _ in reader.read() {}
            writer.write(TestEvent);
        },
        |mut reader: EventReader<TestEvent>| {
            // Another system reading - this will conflict with the writer above
            for _ in reader.read() {}
        },
    ));

    // This SHOULD panic
    app.update();
}
