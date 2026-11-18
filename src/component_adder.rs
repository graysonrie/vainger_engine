use std::marker::PhantomData;

use bevy::ecs::query::QueryFilter;

use crate::prelude::*;

#[derive(Component)]
pub struct ComponentAdder<C: Component + Default> {
    _p: PhantomData<C>,
    current_amount: u32,
    threshold: u32,
}
impl<C: Component + Default> Default for ComponentAdder<C> {
    fn default() -> Self {
        Self::new(1)
    }
}

impl<C: Component + Default> ComponentAdder<C> {
    pub fn new(threshold: u32) -> Self {
        Self {
            _p: PhantomData,
            current_amount: 0,
            threshold,
        }
    }

    pub fn increment(&mut self, entity: Entity, commands: &mut Commands) {
        self.current_amount += 1;
        if self.current_amount >= self.threshold {
            commands.entity(entity).insert(C::default());
        }
    }

    pub fn decrement(&mut self, entity: Entity, commands: &mut Commands) {
        self.current_amount -= 1;
        if self.current_amount < self.threshold {
            commands.entity(entity).remove::<C>();
        }
    }
}

pub fn register<T: Component, C: Component + Default, F: QueryFilter + 'static>(app: &mut App) {
    app.add_observer(
        |event: On<Add, T>,
         mut q: Query<(Entity, &mut ComponentAdder<C>), F>,
         mut commands: Commands| {
            let Ok((entity, mut adder)) = q.get_mut(event.entity) else {
                return;
            };
            adder.increment(entity, &mut commands);
        },
    );

    app.add_observer(
        |event: On<Remove, T>,
         mut q: Query<(Entity, &mut ComponentAdder<C>), F>,
         mut commands: Commands| {
            let Ok((entity, mut adder)) = q.get_mut(event.entity) else {
                return;
            };
            adder.decrement(entity, &mut commands);
        },
    );
}

pub trait AppComponentAdderExt {
    /// The first generic is the component that will make the adder increment if added
    /// and decrement if removed
    ///
    /// The second generic is the component that will get added if the adder crosses the
    /// threshold
    ///
    /// The third generic is the query filter. Ex: `With<Player>`
    fn component_adder_track<T: Component, C: Component + Default, F: QueryFilter + 'static>(
        &mut self,
    ) -> &mut Self;
}

impl AppComponentAdderExt for App {
    fn component_adder_track<T: Component, C: Component + Default, F: QueryFilter + 'static>(
        &mut self,
    ) -> &mut Self {
        register::<T, C, F>(self);
        self
    }
}
