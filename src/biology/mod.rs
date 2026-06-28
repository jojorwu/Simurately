pub mod animal;
pub mod plant;
pub mod genome;

use glam::Vec2;
use self::genome::Genome;

pub trait Entity {
    fn id(&self) -> u64;
    fn position(&self) -> Vec2;
    fn health(&self) -> f32;
    fn age(&self) -> u32;
    fn is_dead(&self) -> bool;
    fn genome(&self) -> &Genome;
}
