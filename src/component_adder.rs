use std::marker::PhantomData;

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
