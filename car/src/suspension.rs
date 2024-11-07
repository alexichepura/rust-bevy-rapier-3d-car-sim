use crate::{WheelMount, CAR_TRAINING_GROUP, STATIC_GROUP};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_suspension(
    cmd: &mut Commands,
    car_space_translation: Vec3,
    car_transform: Transform,
    joint: MultibodyJoint,
) -> Entity {
    let translation =
        car_transform.translation + car_transform.rotation.mul_vec3(car_space_translation);
    let transform = Transform::from_translation(translation);
    // .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI));
    let collider = Collider::cylinder(0.1, 0.05);
    cmd.spawn((
        Name::new("suspension"),
        joint,
        transform,
        collider,
        // Sensor,
        ColliderMassProperties::default(),
        CollisionGroups::new(CAR_TRAINING_GROUP, STATIC_GROUP),
        ColliderScale::Absolute(Vec3::ONE),
        ExternalForce::default(),
        ExternalImpulse::default(),
        RigidBody::Dynamic,
        Sleeping::disabled(),
        Velocity::zero(),
    ))
    // .with_children(|children| {
    //     // NOTE: we want to attach multiple impulse joints to this entity, so
    //     //       we need to add the components to children of the entity. Otherwise
    //     //       the second joint component would just overwrite the first one.
    //     children.spawn(joint);
    // })
    .id()
}
