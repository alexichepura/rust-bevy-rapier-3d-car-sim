pub mod asphalt;
pub mod decor;
pub mod ground;
pub mod kerb;
pub mod track;
pub mod wall;

pub use asphalt::*;
pub use decor::*;
pub use ground::*;
pub use track::*;

use crate::material::MaterialHandle;
use bevy::prelude::*;

use self::{
    asphalt::spawn_road, ground::spawn_ground_heightfield, kerb::spawn_kerb, track::Track,
    wall::spawn_walls,
};

pub fn track_start_system(
    handled_materials: Res<MaterialHandle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let track = Track::new();
    let aabb = spawn_road(&handled_materials, &mut commands, &mut meshes, &track);
    spawn_ground_heightfield(&mut commands, &mut meshes, &handled_materials, &aabb, 100.);

    spawn_kerb(&mut commands, &mut meshes, &handled_materials, &track);
    let mut left_wall_points: Vec<Vec3> = vec![];
    let mut right_wall_points: Vec<Vec3> = vec![];
    for (i, p) in track.points.iter().enumerate() {
        left_wall_points.push(*p + track.right_norm[i] * 7.5);
        right_wall_points.push(*p + track.right_norm[i] * -7.5);
    }
    spawn_walls(
        &mut commands,
        &mut meshes,
        &handled_materials,
        &track.indices,
        &left_wall_points,
        &track.right_norm,
    );
    spawn_walls(
        &mut commands,
        &mut meshes,
        &handled_materials,
        &track.indices,
        &right_wall_points,
        &track.right_norm,
    );
}
