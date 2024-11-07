cfg_if::cfg_if! {if #[cfg(feature = "graphics")] {
    pub mod res;
    pub mod sensor;
    pub use res::CarRes;
}}

pub mod car;
pub mod esp;
pub mod joint;
pub mod spawn;
pub mod spec;
pub mod suspension;
pub mod wheel;

pub use car::*;
pub use esp::*;
pub use spec::*;
pub use suspension::*;
pub use wheel::*;

use bevy::prelude::SystemSet;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum CarSet {
    Input,
    NeuralNetwork,
    Esp,
}
