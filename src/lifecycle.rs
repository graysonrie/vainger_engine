
use crate::prelude::*;

pub struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, GameRestart::handle_read);
    }
}

#[derive(Component)]
pub struct GameManaged;

#[derive(Message)]
pub struct GameRestart;

#[derive(Event)]
pub struct AlertResourceRestart;

#[derive(Event)]
pub struct RestartProcessComplete;

impl GameRestart {
    fn handle_read(mut reader: MessageReader<Self>, mut commands: Commands) {
        if reader.is_empty() {
            return;
        }
        reader.clear();
        commands.trigger(AlertResourceRestart);

        commands.trigger(RestartProcessComplete);
    }
}
