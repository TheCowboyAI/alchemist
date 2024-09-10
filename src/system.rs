#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

fn print_position_system(query: Query<&Position>) {
  for position in &query {
      println!("position: {} {}", position.x, position.y);
  }
}

struct Entity(u64);
