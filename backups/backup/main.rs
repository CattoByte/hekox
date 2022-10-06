use bevy::prelude::*;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Name(String);

fn list_players(query: Query<&Name, With<Player>>) {
    println!("Current players:");
    for name in query.iter() {
        println!("\t{}", name.0);
    }
}

fn add_player(mut commands: Commands) {
    commands
        .spawn()
        .insert(Player)
        .insert(Name("CattoByte".to_string()));
    commands
        .spawn()
        .insert(Player)
        .insert(Name("Axy0C".to_string()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(add_player)
        .add_system(list_players)
        .run();
}
