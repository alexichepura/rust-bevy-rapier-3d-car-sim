use bevy::prelude::*;
use bevy_rapier3d::parry::shape::Polyline;
use rand::Rng;
// use std::f32::consts::PI;

#[derive(Resource)]
pub struct TrackConfig {
    pub polyline: Option<Polyline>,
    pub segments: Vec<f32>,
    pub start_segment_i: usize,
    pub start_segment_shift: f32,
    pub start_shift: f32,
    pub track_length: f32,
}
impl Default for TrackConfig {
    fn default() -> Self {
        Self {
            polyline: None,
            segments: vec![],
            start_segment_i: 0,
            start_segment_shift: 0.,
            start_shift: 0.,
            track_length: 0.,
        }
    }
}
impl TrackConfig {
    // pub fn get_transform_by_index(&self, i: usize) -> (Transform, f32) {
    //     let meters = i as f32 * self.track_length / self.cars_count as f32;
    //     let (translate, quat) = self.get_transform_by_meter(meters);
    //     let transform = Transform::from_translation(translate).with_rotation(quat);
    //     return (transform, meters);
    // }
    pub fn get_transform_random(&self) -> (Transform, f32) {
        let mut rng = rand::thread_rng();
        let meters = rng.gen_range(0.0..self.track_length);
        let (translate, quat) = self.get_transform_by_meter(meters);
        let transform = Transform::from_translation(translate).with_rotation(quat);
        return (transform, meters);
    }
    pub fn get_transform_by_meter(&self, meters: f32) -> (Vec3, Quat) {
        let polyline = self.polyline.as_ref().unwrap();
        let mut seg_meters = 0.;
        let mut shift = meters + self.start_shift;
        if shift > self.track_length {
            shift = shift - self.track_length * (shift / self.track_length).floor();
        }

        for segment in polyline.segments() {
            let new_seg_meters: f32 = seg_meters + segment.length();
            if new_seg_meters < shift {
                seg_meters = new_seg_meters;
            } else {
                let a: Vec3 = segment.a.into();
                let dir: Vec3 = segment.direction().unwrap().into();
                let mut pos: Vec3 = a + dir * (shift - seg_meters);
                pos.y = 0.57;

                return (pos, Quat::from_rotation_arc(Vec3::Z, dir));
            }
        }
        panic!();
    }
}
