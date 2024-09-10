#[derive(Resource)]
struct UiFont(Handle<Font>);

fn load_Uifont(
    mut commands: Commands,
    server: Res<AssetServer>
) {
    let handle: Handle<Font> = server.load("fonts/FiraCodeNerdFont-Regular.ttf");

    // we can store the handle in a resource:
    //  - to prevent the asset from being unloaded
    //  - if we want to use it to access the asset later
    commands.insert_resource(UiFont(handle));
}