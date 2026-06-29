pub mod animal;
pub mod plant;
pub mod genome;
pub mod ai;

use glam::Vec2;
use self::genome::Genome;

/// Common interface for all biological entities in the simulation.
pub trait Entity {
    /// Unique identifier of the entity.
    fn id(&self) -> u64;
    /// Current position of the entity in world space.
    fn position(&self) -> Vec2;
    /// Current health points of the entity.
    fn health(&self) -> f32;
    /// Current age of the entity in simulation ticks.
    fn age(&self) -> u32;
    /// Returns true if the entity's health is zero or it has reached its maximum lifespan.
    fn is_dead(&self) -> bool;
    /// Returns a reference to the entity's genome.
    fn genome(&self) -> &Genome;
}
