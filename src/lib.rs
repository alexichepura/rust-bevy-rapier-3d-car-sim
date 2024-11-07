mod config;
mod dash;
pub mod font;
mod input;
#[cfg(feature = "virtual_joystick")]
pub mod joystick;
mod spawn;
use std::num::NonZeroUsize;

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, ecs::system::SystemParam,
    pbr::DirectionalLightShadowMap, prelude::*,
};
use bevy_garage_car::{aero_system, car_start_system, esp_system, CarRes, CarSet};
use bevy_garage_light::{animate_light_direction, light_start_system};
use bevy_garage_track::{track_polyline_start_system, SpawnCarOnTrackEvent, TrackPlugin};
use bevy_rapier3d::prelude::*;
use config::*;
use dash::*;
use font::*;
use input::*;
use spawn::*;

fn rapier_config_start_system(mut c: ResMut<RapierContext>) {
    c.integration_parameters.num_solver_iterations = NonZeroUsize::new(6).unwrap();
    c.integration_parameters.warmstart_coefficient = 0.;
    c.integration_parameters.contact_natural_frequency = 50.;
    c.integration_parameters.contact_damping_ratio = 50.;
    // c.integration_parameters.joint_damping_ratio = 0.1;
    // c.integration_parameters.joint_natural_frequency = 1e2;
    // c.integration_parameters.num_internal_pgs_iterations = 48;
    c.integration_parameters.num_additional_friction_iterations = 4;
    dbg!(c.integration_parameters);
}

pub fn car_app(app: &mut App) -> &mut App {
    #[cfg(feature = "nn")]
    let esp_run_after: CarSet = CarSet::NeuralNetwork;
    #[cfg(not(feature = "nn"))]
    let esp_run_after: CarSet = CarSet::Input;

    let mut rapier_config = RapierConfiguration::new(1.);
    rapier_config.timestep_mode = TimestepMode::Variable {
        max_dt: 1. / 60.,
        time_scale: 1.,
        substeps: 5,
    };
    app.init_resource::<FontHandle>()
        .insert_resource(rapier_config)
        .insert_resource(Msaa::Sample4)
        .insert_resource(Config::default())
        .insert_resource(CarRes::default())
        .insert_resource(DirectionalLightShadowMap::default())
        .add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            RapierPhysicsPlugin::<MyPhysicsHooks>::default(),
            TrackPlugin,
            RapierDebugRenderPlugin {
                enabled: false,
                style: DebugRenderStyle {
                    rigid_body_axes_length: 0.5,
                    ..default()
                },
                mode: DebugRenderMode::COLLIDER_SHAPES
                    | DebugRenderMode::RIGID_BODY_AXES
                    | DebugRenderMode::JOINTS
                    | DebugRenderMode::CONTACTS
                    | DebugRenderMode::SOLVER_CONTACTS,
                ..default()
            },
        ))
        .add_event::<SpawnCarOnTrackEvent>()
        .add_systems(
            Startup,
            (
                car_start_system.after(track_polyline_start_system),
                spawn_car_start_system.after(car_start_system),
                light_start_system,
                dash_start_system,
                rapier_config_start_system,
            ),
        )
        .add_systems(
            Update,
            (
                spawn_car_system,
                aero_system.in_set(CarSet::Input),
                input_system.in_set(CarSet::Input),
                esp_system.in_set(CarSet::Esp).after(esp_run_after),
                animate_light_direction,
                dash_fps_system,
                dash_speed_update_system,
            ),
        );

    #[cfg(feature = "dsp")]
    {
        app.add_plugins(bevy_garage_dsp::EngineSoundPlugin);
    }
    #[cfg(feature = "nn")]
    {
        app.add_plugins(bevy_garage_nn::NeuralNetworkPlugin);
    }

    app
}

#[derive(SystemParam)]
struct MyPhysicsHooks;

impl BevyPhysicsHooks for MyPhysicsHooks {
    fn modify_solver_contacts(&self, context: ContactModificationContextView) {
        // *context.raw.normal = -*context.raw.normal;
        // println!("normal {:?}", context.raw.bodies);
        // if !context.raw.solver_contacts.is_empty() {
        //     context.raw.solver_contacts.swap_remove(0);
        // }
        // let manifold = context.raw.manifold;
        // manifold.data;
        // println!("solver_contacts={:?}", &context.raw.solver_contacts);
        for solver_contact in &mut *context.raw.solver_contacts {
            // println!("solver {:?}", solver_contact.tangent_velocity);
            // solver_contact.warmstart_impulse = 0.0;
            // solver_contact.warmstart_tangent_impulse =
            // bevy_rapier3d::rapier::math::TangentImpulse::zeros();

            // solver_contact.friction = 0.3;
            // solver_contact.restitution = 0.1;
            // solver_contact.tangent_velocity.x = 10.0;
        }

        // Use the persistent user-data to count the number of times
        // contact modification was called for this contact manifold
        // since its creation.
        // *context.raw.user_data += 1;
        // println!(
        //     "Contact manifold has been modified {} times since its creation.",
        //     *context.raw.user_data
        // );
    }
}
