use std::any::TypeId;

use bevy::ecs::{component::Mutable, schedule::ScheduleLabel};
use thiserror::Error;

use crate::prelude::*;

/// Opt-in restart for a running game.
///
/// Add this plugin in every build. [`GameStart`] runs once from [`Startup`], and again
/// whenever a [`GameRestart`] message is written. In debug builds, pressing Alt writes that
/// message. The hotkey is compiled out of release builds.
///
/// Engine objects stay put. Mark game entities with [`GameManaged`], and register game
/// resources with [`RestartAppExt::insert_restartable_resource`]. Leave resources such as
/// `Time`, the camera, and asset storage unregistered.
pub struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RestartRegistry>()
            .add_message::<GameRestart>()
            .init_schedule(GameStart)
            .add_systems(Startup, run_game_start)
            .add_systems(PreUpdate, apply_restart);

        #[cfg(debug_assertions)]
        app.add_systems(
            PreUpdate,
            hotkey_restart
                .after(bevy::input::InputSystems)
                .before(apply_restart)
                .run_if(resource_exists::<ButtonInput<KeyCode>>),
        );
    }
}

/// Marker for entities that despawn when the game restarts.
///
/// Despawn also removes descendants, so a marked root takes its children with it.
/// Marking a child as well is safe.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GameManaged;

/// Ask [`LifecyclePlugin`] to restart the game.
///
/// Debug builds write this when Alt is pressed. Game code and tests can write it too.
#[derive(Message, Debug, Clone, Copy)]
pub struct GameRestart;

/// Fired after a restart has despawned [`GameManaged`] entities, restored resources,
/// and run [`GameStart`]. Not fired for the first boot.
#[derive(Event, Debug, Clone, Copy)]
pub struct GameRestarted;

/// Systems that build the game session.
///
/// They run once on startup and again after each restart, after resources have been
/// restored. Put one-time setup (the window, asset loading, the camera) in [`Startup`]
/// instead.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameStart;

/// Registers resources that [`LifecyclePlugin`] restores on restart.
pub trait RestartAppExt {
    /// Insert `value` and keep a clone as the baseline restored on restart.
    ///
    /// A second call for the same resource replaces that baseline and keeps a single restorer.
    /// [`GameStart`] then runs and may mutate the resource again.
    fn insert_restartable_resource<T: Resource + Clone>(&mut self, value: T) -> &mut Self;

    /// Restore `T` from a closure on restart.
    ///
    /// Use this for resources that are not [`Clone`]. Capture the original values in `reset`.
    /// A later call for the same resource, including [`insert_restartable_resource`](Self::insert_restartable_resource),
    /// replaces this restorer.
    fn reset_on_restart<T>(&mut self, reset: impl Fn(&mut T) + Send + Sync + 'static) -> &mut Self
    where
        T: Resource<Mutability = Mutable>;
}

impl RestartAppExt for App {
    fn insert_restartable_resource<T: Resource + Clone>(&mut self, value: T) -> &mut Self {
        self.init_resource::<RestartRegistry>();
        self.insert_resource(value.clone());
        self.insert_resource(RestartSnapshot(value));
        register_restorer(
            self.world_mut(),
            TypeId::of::<T>(),
            Box::new(restore_snapshot::<T>),
        );
        self
    }

    fn reset_on_restart<T>(&mut self, reset: impl Fn(&mut T) + Send + Sync + 'static) -> &mut Self
    where
        T: Resource<Mutability = Mutable>,
    {
        self.init_resource::<RestartRegistry>();
        register_restorer(
            self.world_mut(),
            TypeId::of::<T>(),
            Box::new(move |world: &mut World| {
                let Some(mut resource) = world.get_resource_mut::<T>() else {
                    return;
                };
                reset(&mut resource);
            }),
        );
        self
    }
}

/// Why [`capture_restart_snapshot`] could not replace a restart baseline.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum RestartSnapshotError {
    /// [`RestartAppExt::insert_restartable_resource`] was never called for this resource.
    #[error("resource was not registered with insert_restartable_resource")]
    NotRegistered,
    /// The resource is registered, but it is not currently in the world.
    #[error("registered resource is missing from the world")]
    MissingResource,
}

/// Replace the stored baseline with a clone of the resource's current value.
///
/// Call this once, when the resource first reaches the state a restart should return to
/// (for example after assets finish loading). It is not a per-frame system.
pub fn capture_restart_snapshot<T: Resource + Clone>(
    world: &mut World,
) -> Result<(), RestartSnapshotError> {
    if !world.contains_resource::<RestartSnapshot<T>>() {
        return Err(RestartSnapshotError::NotRegistered);
    }
    let Some(current) = world.get_resource::<T>().cloned() else {
        return Err(RestartSnapshotError::MissingResource);
    };
    world.resource_mut::<RestartSnapshot<T>>().0 = current;
    Ok(())
}

#[derive(Resource)]
struct RestartSnapshot<T: Resource>(T);

struct Restorer {
    type_id: TypeId,
    apply: Box<dyn Fn(&mut World) + Send + Sync>,
}

#[derive(Resource, Default)]
struct RestartRegistry {
    restorers: Vec<Restorer>,
}

fn register_restorer(
    world: &mut World,
    type_id: TypeId,
    apply: Box<dyn Fn(&mut World) + Send + Sync>,
) {
    let mut registry = world.resource_mut::<RestartRegistry>();
    if let Some(existing) = registry
        .restorers
        .iter_mut()
        .find(|restorer| restorer.type_id == type_id)
    {
        existing.apply = apply;
    } else {
        registry.restorers.push(Restorer { type_id, apply });
    }
}

fn restore_snapshot<T: Resource + Clone>(world: &mut World) {
    let Some(value) = world
        .get_resource::<RestartSnapshot<T>>()
        .map(|snapshot| snapshot.0.clone())
    else {
        return;
    };
    world.insert_resource(value);
}

fn run_game_start(world: &mut World) {
    world.run_schedule(GameStart);
}

#[cfg(debug_assertions)]
fn hotkey_restart(keys: Res<ButtonInput<KeyCode>>, mut restart: MessageWriter<GameRestart>) {
    if keys.just_pressed(KeyCode::AltLeft) || keys.just_pressed(KeyCode::AltRight) {
        restart.write(GameRestart);
    }
}

fn apply_restart(world: &mut World) {
    {
        let mut messages = world.resource_mut::<Messages<GameRestart>>();
        if messages.is_empty() {
            return;
        }
        messages.clear();
    }

    info!("Restarting game");

    let managed: Vec<Entity> = world
        .iter_entities()
        .filter(|entity| entity.contains::<GameManaged>())
        .map(|entity| entity.id())
        .collect();
    for entity in managed {
        // A marked parent may already have despawned this entity with its descendants.
        let _ = world.try_despawn(entity);
    }

    // Take the list so each restorer can mutably access the world.
    let restorers = std::mem::take(&mut world.resource_mut::<RestartRegistry>().restorers);
    for restorer in &restorers {
        (restorer.apply)(world);
    }
    world.resource_mut::<RestartRegistry>().restorers = restorers;

    world.run_schedule(GameStart);
    world.trigger(GameRestarted);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource, Clone, PartialEq, Debug)]
    struct Score(i32);

    #[derive(Resource, Default)]
    struct SpawnCount(u32);

    #[derive(Resource, Default)]
    struct RestartCount(u32);

    #[derive(Resource, PartialEq, Debug)]
    struct Session {
        score: i32,
    }

    fn spawn_world(mut commands: Commands, mut count: ResMut<SpawnCount>) {
        commands.spawn(GameManaged);
        count.0 += 1;
    }

    #[test]
    fn restart_despawns_managed_entities_and_restores_resources() {
        let mut app = App::new();
        app.add_plugins(LifecyclePlugin)
            .insert_restartable_resource(Score(1))
            .init_resource::<SpawnCount>()
            .init_resource::<RestartCount>()
            .add_systems(GameStart, spawn_world)
            .add_observer(|_: On<GameRestarted>, mut count: ResMut<RestartCount>| {
                count.0 += 1;
            });

        let unmarked = app.world_mut().spawn_empty().id();
        let marked = app.world_mut().spawn(GameManaged).id();
        let child = app.world_mut().spawn_empty().id();
        app.world_mut().entity_mut(marked).add_child(child);

        app.update();

        assert_eq!(app.world().resource::<SpawnCount>().0, 1);
        assert_eq!(app.world().resource::<RestartCount>().0, 0);
        assert!(app.world().get_entity(marked).is_ok());
        assert!(app.world().get_entity(child).is_ok());

        app.world_mut().resource_mut::<Score>().0 = 99;
        app.world_mut()
            .resource_mut::<Messages<GameRestart>>()
            .write(GameRestart);

        app.update();

        assert!(app.world().get_entity(marked).is_err());
        assert!(app.world().get_entity(child).is_err());
        assert!(app.world().get_entity(unmarked).is_ok());
        assert_eq!(app.world().resource::<Score>().0, 1);
        assert_eq!(app.world().resource::<SpawnCount>().0, 2);
        assert_eq!(app.world().resource::<RestartCount>().0, 1);

        let mut managed = app
            .world_mut()
            .query_filtered::<Entity, With<GameManaged>>();
        let managed_entities: Vec<_> = managed.iter(app.world()).collect();
        assert_eq!(managed_entities.len(), 1);
        assert_ne!(managed_entities[0], marked);
    }

    #[test]
    fn capture_and_manual_reset_restore_their_baselines() {
        let mut app = App::new();
        app.add_plugins(LifecyclePlugin);

        let not_registered = capture_restart_snapshot::<Score>(app.world_mut());
        assert_eq!(not_registered, Err(RestartSnapshotError::NotRegistered));

        app.insert_restartable_resource(Score(0))
            .insert_restartable_resource(Score(1))
            .insert_resource(Session { score: 0 })
            .reset_on_restart::<Session>(|session| session.score = 3);

        app.world_mut().remove_resource::<Score>();
        let missing = capture_restart_snapshot::<Score>(app.world_mut());
        assert_eq!(missing, Err(RestartSnapshotError::MissingResource));
        app.world_mut().insert_resource(Score(1));

        app.update();

        app.world_mut().resource_mut::<Score>().0 = 6;
        capture_restart_snapshot::<Score>(app.world_mut()).unwrap();
        app.world_mut().resource_mut::<Score>().0 = 8;
        app.world_mut().resource_mut::<Session>().score = 9;
        app.world_mut()
            .resource_mut::<Messages<GameRestart>>()
            .write(GameRestart);

        app.update();

        assert_eq!(app.world().resource::<Score>().0, 6);
        assert_eq!(app.world().resource::<Session>().score, 3);
    }
}
