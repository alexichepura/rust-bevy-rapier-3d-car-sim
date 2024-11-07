use bevy::{color::palettes::css, prelude::*};
use bevy_rapier3d::prelude::*;
use std::f32::consts::PI;

use crate::{Car, CarSpec, CarWheels, Wheel};

pub fn aero_system(mut car_query: Query<(&Velocity, &Transform, &mut ExternalForce), With<Car>>) {
    for (velocity, transform, mut force) in car_query.iter_mut() {
        let car_vector = transform.rotation.mul_vec3(Vec3::Z);
        let car_vector_norm = car_vector.normalize();
        let car_mps = velocity.linvel.length();
        let f_drag = 1. / 2. * 1.2 * car_mps.powi(2) * 0.2 * 1.5;
        let f_down = car_mps.powi(2) * 2.;
        // println!("drag:{f_drag:.1} down:{f_down:.1}");
        force.force = -Vec3::Y * f_down - car_vector_norm * f_drag;
    }
}

// Quat::from_axis_angle(-Vec3::Y, PI / 2.) = Quat(-0.0, -0.70710677, -0.0, 0.70710677);
const WHEEL_RAY_END_QUAT: Quat = Quat::from_xyzw(-0.0, -0.70710677, -0.0, 0.70710677);
const WHEEL_RAY_SHIFT: Vec3 = Vec3 {
    x: 0.,
    y: 0.5,
    z: 0.,
};

pub fn esp_system(
    time: Res<Time>,
    mut car_query: Query<(&mut Car, &CarSpec, &CarWheels, &Velocity, &Transform)>,
    mut wheels_query: Query<(
        &Wheel,
        &mut ExternalForce,
        &Transform,
        &Velocity,
        &mut MultibodyJoint,
    )>,
    #[cfg(feature = "graphics")] car_res: Res<crate::CarRes>,
    #[cfg(feature = "graphics")] mut gizmos: Gizmos,
) {
    let d_seconds = time.delta_seconds();
    for (mut car, spec, car_wheels, velocity, transform) in car_query.iter_mut() {
        let car_vector = transform.rotation.mul_vec3(Vec3::Z);
        let car_vector_norm = car_vector.normalize();
        let delta = velocity.linvel.normalize() - car_vector_norm;
        let car_angle_slip_rad = Vec3::new(delta.x, 0., delta.z).length();
        let moving_forward: bool = car_angle_slip_rad < PI / 2.;
        let braking = match moving_forward {
            true => car.brake > 0.,
            false => car.gas > 0.,
        };
        let linvel = velocity.linvel.length();
        let torque_speed_x: f32 = match braking {
            true => 2.,
            _ => match linvel / spec.max_speed {
                x if x >= 1. => 0.,
                x => 0.7 + 0.3 * (1. - x),
            },
        };
        let steering_speed_x: f32 = match linvel / spec.max_steering_speed {
            x if x >= 1. => 0.,
            x => 1. - x,
        }
        .powi(2);
        let pedal = if moving_forward {
            if braking {
                -car.brake
            } else {
                car.gas
            }
        } else {
            if braking {
                car.gas
            } else {
                -car.brake
            }
        };
        let dir = pedal.signum();
        let is_same_dir = car.prev_dir == dir;
        let car_torque = pedal.abs() * spec.wheel_max_torque;
        let prev_torque = if is_same_dir { car.prev_torque } else { 0. };
        let prev_steering = car.prev_steering;
        let (steering, mut torque) = (
            prev_steering + (car.steering - prev_steering) * d_seconds * 5.,
            prev_torque + (car_torque - prev_torque) * d_seconds * 10.,
        );
        car.prev_steering = steering;
        car.prev_torque = torque;
        car.prev_dir = dir;

        torque = dir * torque;

        let angle: f32 = spec.wheel_max_angle * steering * (0.1 + 0.9 * steering_speed_x);
        let quat = -Quat::from_axis_angle(Vec3::Y, -angle);
        let torque_vec = Vec3::new(0., torque, 0.);
        let steering_torque_vec = quat.mul_vec3(torque_vec);

        // dbg!(car_wheels.entities);
        for wheel_entity in car_wheels.entities.iter() {
            let (wheel, mut f, transform, v, mut j) = wheels_query.get_mut(*wheel_entity).unwrap();
            let radius_vel = v.angvel * wheel.radius;
            let velocity_slip = (radius_vel[0] - v.linvel[2], radius_vel[2] + v.linvel[0]);
            let slip_sq = (velocity_slip.0.powi(2) + velocity_slip.1.powi(2)).sqrt();
            if wheel.front {
                let max_slip = 50.;
                let slip_sq_x: f32 = match slip_sq / max_slip {
                    x if x >= 1. => 0.,
                    x => 1. - x,
                };
                let total_torque = steering_torque_vec * slip_sq_x * torque_speed_x;
                let wheel_torque = if wheel.left {
                    -total_torque
                } else {
                    total_torque
                };
                f.torque = (transform.rotation.mul_vec3(wheel_torque)).into();

                #[cfg(feature = "graphics")]
                if car_res.show_rays {
                    let start = transform.translation + WHEEL_RAY_SHIFT;
                    let end = start + WHEEL_RAY_END_QUAT.mul_vec3(f.torque) / 200.;
                    gizmos.line(start, end, css::VIOLET);
                }

                j.data.as_mut().set_local_basis1(quat);
            } else {
                let max_slip = 5.;
                let slip_sq_x: f32 = match slip_sq / max_slip {
                    x if x >= 1. => 0.,
                    x => 1. - x,
                };
                let total_torque = torque_vec * slip_sq_x * torque_speed_x;
                let wheel_torque = if wheel.left {
                    -total_torque
                } else {
                    total_torque
                };
                f.torque = (transform.rotation.mul_vec3(wheel_torque)).into();

                #[cfg(feature = "graphics")]
                if car_res.show_rays {
                    let start = transform.translation + WHEEL_RAY_SHIFT;
                    let end = start + WHEEL_RAY_END_QUAT.mul_vec3(f.torque) / 200.;
                    gizmos.line(start, end, Color::BLACK);
                }
            }
        }
    }
}
