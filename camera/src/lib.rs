use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy_garage_car::Player;
use bevy_rapier3d::prelude::PhysicsSet;

pub fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

pub struct CarCameraPlugin;

impl Plugin for CarCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraConfig::default())
            .add_systems(PostStartup, camera_start_system)
            .add_systems(Update, (grab_mouse, camera_switch_system))
            .add_systems(
                PostUpdate,
                camera_controller_system
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystem::TransformPropagate),
            );
    }
}

pub fn camera_start_system(mut cmd: Commands) {
    let sky_blue: Color = Srgba::hex("87CEEB").unwrap().into();
    cmd.spawn((
        Camera3dBundle {
            #[cfg(not(any(target_arch = "wasm32", target_os = "ios", target_os = "android")))]
            projection: Projection::from(PerspectiveProjection {
                far: 5000.,
                near: 0.01,
                ..default()
            }),
            #[cfg(any(target_arch = "wasm32", target_os = "ios", target_os = "android"))]
            projection: Projection::from(PerspectiveProjection {
                far: 50.,
                near: 0.1,
                ..default()
            }),
            #[cfg(any(target_os = "ios"))]
            dither: bevy::core_pipeline::tonemapping::DebandDither::Disabled,
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        FogSettings {
            color: sky_blue, // Color::rgba(0.1, 0.2, 0.4, 1.0),
            directional_light_color: Color::srgba(1.0, 0.95, 0.75, 1.),
            directional_light_exponent: 200.0,
            falloff: FogFalloff::from_visibility_colors(
                5000.,
                Color::srgb(0.35, 0.5, 0.66),
                Color::srgb(0.8, 0.844, 1.0),
            ),
        },
        CameraController::default(),
    ));
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.5,
            key_forward: KeyCode::KeyW,
            key_back: KeyCode::KeyS,
            key_left: KeyCode::KeyA,
            key_right: KeyCode::KeyD,
            key_up: KeyCode::KeyE,
            key_down: KeyCode::KeyQ,
            key_run: KeyCode::ShiftLeft,
            walk_speed: 2.0,
            run_speed: 100.0,
            friction: 0.8,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum CameraFollowView {
    Windshield,
    FrontWheel,
    Driver,
    Near,
    Mid,
    Far,
}
fn follow_props_by_mode(mode: &CameraFollowView) -> (Vec3, Vec3) {
    let look_from = match mode {
        CameraFollowView::Windshield => Vec3::new(0., -0.85, 0.26),
        CameraFollowView::Driver => Vec3::new(0., 0.47, -0.402),
        CameraFollowView::Near => Vec3::new(0., 2., -5.),
        CameraFollowView::Mid => Vec3::new(0., 3., -10.),
        CameraFollowView::Far => Vec3::new(0., 5., -20.),
        CameraFollowView::FrontWheel => Vec3::new(-2.5, -0.2, 2.),
    };
    let look_at = match mode {
        CameraFollowView::Windshield => Vec3::new(0., 0., 0.), // TODO
        CameraFollowView::Driver => Vec3::new(0., 0.47, 0.),
        CameraFollowView::Near => Vec3::new(0., 1.5, 0.),
        CameraFollowView::Mid => Vec3::new(0., 2., 0.),
        CameraFollowView::Far => Vec3::new(0., 3., 0.),
        CameraFollowView::FrontWheel => Vec3::new(0., -0.2, 1.),
    };
    (look_from, look_at)
}
#[derive(PartialEq, Debug)]
pub enum CameraMode {
    Follow(CameraFollowView, Vec3, Vec3),
    Free,
}

#[derive(Resource)]
pub struct CameraConfig {
    pub mode: CameraMode,
    pub prev: Transform,
}

impl CameraConfig {
    pub fn from_view(view: CameraFollowView) -> Self {
        let (from, at) = follow_props_by_mode(&view);
        Self {
            mode: CameraMode::Follow(view, from, at),
            prev: Transform::IDENTITY,
        }
    }
    pub fn next_view(&mut self) {
        let mode = match self.mode {
            CameraMode::Follow(ref view, _, _) => {
                let next_view: CameraFollowView = match view {
                    CameraFollowView::Windshield => CameraFollowView::Driver,
                    CameraFollowView::Driver => CameraFollowView::Near,
                    CameraFollowView::Near => CameraFollowView::Mid,
                    CameraFollowView::Mid => CameraFollowView::Far,
                    CameraFollowView::Far => CameraFollowView::FrontWheel,
                    CameraFollowView::FrontWheel => CameraFollowView::Windshield,
                };

                let (from, at) = follow_props_by_mode(&next_view);
                CameraMode::Follow(next_view, from, at)
            }
            CameraMode::Free => CameraMode::Free,
        };
        self.mode = mode;
    }
    pub fn free(&mut self) {
        self.mode = CameraMode::Free;
    }
    fn follow_view(&mut self, view: CameraFollowView) {
        let (from, at) = follow_props_by_mode(&view);
        self.mode = CameraMode::Follow(view, from, at);
    }
    pub fn driver(&mut self) {
        self.follow_view(CameraFollowView::Driver);
    }
    pub fn near(&mut self) {
        self.follow_view(CameraFollowView::Near);
    }
    pub fn mid(&mut self) {
        self.follow_view(CameraFollowView::Mid);
    }
    pub fn far(&mut self) {
        self.follow_view(CameraFollowView::Far);
    }
    pub fn wheel(&mut self) {
        self.follow_view(CameraFollowView::FrontWheel);
    }
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self::from_view(CameraFollowView::Near)
    }
}
pub fn camera_switch_system(mut config: ResMut<CameraConfig>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        config.driver();
    }
    if input.just_pressed(KeyCode::Digit2) {
        config.near();
    }
    if input.just_pressed(KeyCode::Digit3) {
        config.mid();
    }
    if input.just_pressed(KeyCode::Digit4) {
        config.far();
    }
    if input.just_pressed(KeyCode::Digit5) {
        config.wheel();
    }
    if input.just_pressed(KeyCode::Digit0) {
        config.free();
    }
}

pub fn camera_controller_system(
    time: Res<Time>,
    config: Res<CameraConfig>,
    mut mouse_events: EventReader<MouseMotion>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut pset: ParamSet<(
        Query<(&mut Transform, &mut CameraController), With<Camera>>,
        Query<&Transform, With<Player>>,
        Query<&mut Transform, With<DirectionalLight>>,
    )>,
    windows: Query<&Window>,
) {
    let follow_option: Option<Transform> = match config.mode {
        CameraMode::Free => None,
        CameraMode::Follow(_, from, at) => {
            if let Ok(car_tf) = pset.p1().get_single() {
                let mut tf = car_tf.clone();
                tf.translation += tf.rotation.mul_vec3(from);
                // tf.rotate_local_y(std::f32::consts::PI);
                tf.look_at(car_tf.translation + tf.rotation.mul_vec3(at), *tf.local_y());
                // tf.look_at(car_tf.translation + tf.rotation.mul_vec3(at), Vec3::Y);
                Some(tf)
            } else {
                None
            }
        }
    };
    let tf: Transform = if let Some(tf) = follow_option {
        let mut p0 = pset.p0();
        let (_, mut options) = p0.single_mut();
        let (yaw, pitch, _roll) = tf.rotation.to_euler(EulerRot::YXZ);
        options.pitch = pitch;
        options.yaw = yaw;
        tf
    } else {
        let window = windows.single();
        if window.cursor.grab_mode == CursorGrabMode::None {
            return;
        }
        let dt = time.delta_seconds();

        let mut mouse_delta = Vec2::ZERO;
        for mouse_event in mouse_events.read() {
            mouse_delta += mouse_event.delta;
        }

        let mut p0 = pset.p0();
        let (tf, mut options) = p0.single_mut();

        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }

        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };
            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }

        let mut tf = tf.clone();
        let forward = *tf.forward();
        let right = *tf.right();
        tf.translation += options.velocity.x * dt * right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * forward;

        if mouse_delta != Vec2::ZERO {
            let (pitch, yaw) = (
                (options.pitch - mouse_delta.y * 0.5 * options.sensitivity * dt).clamp(
                    -0.99 * std::f32::consts::FRAC_PI_2,
                    0.99 * std::f32::consts::FRAC_PI_2,
                ),
                options.yaw - mouse_delta.x * options.sensitivity * dt,
            );
            tf.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            options.pitch = pitch;
            options.yaw = yaw;
        }
        tf
    };
    let mut p0 = pset.p0();
    let (mut camera_tf, _) = p0.single_mut();
    camera_tf.translation = tf.translation;
    camera_tf.rotation = tf.rotation;
}
