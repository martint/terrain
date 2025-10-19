use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::light::AtmosphereEnvironmentMapLight;
use bevy::light::light_consts::lux;
use bevy::pbr::Atmosphere;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use std::f32::consts::PI;

mod camera;
mod camera_widget;
mod terrain;

use camera_widget::{CameraWidgetPlugin, MainCamera};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WireframePlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default()) // Add FPS diagnostics
        .add_plugins(CameraWidgetPlugin)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1.0, 1.0, 0.0), // Yellow wireframe
        })
        .init_resource::<terrain::TerrainManager>()
        .add_systems(
            Startup,
            (
                setup,
                terrain::setup_terrain,
                setup_ui,
                setup_cursor,
            ),
        )
        .add_systems(
            Update,
            (
                camera::toggle_cursor,
                camera::camera_movement,
                terrain::toggle_wireframe_system,
                terrain::toggle_normals_system,
                update_ui_system,
            ),
        )
        .add_systems(Update, dynamic_scene)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Atmosphere::EARTH,
        Exposure::SUNLIGHT,
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
        Tonemapping::AcesFitted,
        camera::CameraController::default(),
        Transform::from_xyz(45.0, 485.0, 680.0).looking_to(Dir3::NEG_Z, Dir3::Y),
        MainCamera,
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: false,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component)]
struct CoordinateText;

fn setup_ui(mut commands: Commands) {
    // Root UI node
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::default(),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                CoordinateText,
            ));
        });
}

fn setup_cursor(mut cursor_options: Single<&mut CursorOptions>) {
    cursor_options.visible = false;
    cursor_options.grab_mode = CursorGrabMode::Locked;
}

fn update_ui_system(
    mut text_query: Query<&mut Text, With<CoordinateText>>,
    camera_query: Query<&Transform, (With<Camera3d>, With<MainCamera>)>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
        .unwrap_or(0.0);

    // Get camera transform
    if let Ok(camera_transform) = camera_query.single() {
        let pos = camera_transform.translation;
        let forward = camera_transform.forward();

        let pitch = forward.y.asin().to_degrees();

        let heading = forward.x.atan2(-forward.z).to_degrees();
        let heading = if heading < 0.0 {
            heading + 360.0
        } else {
            heading
        };

        text.0 = format!(
            "FPS: {:.1}\n\nCoord: ({:.1},{:.1},{:.1})\nPitch: {:.1} deg\nHeading: {:.1} deg",
            fps, pos.x, pos.y, pos.z, pitch, heading
        );
    } else {
        text.0 = format!("FPS: {:.1}\n", fps);
    }
}

fn dynamic_scene(mut suns: Query<&mut Transform, With<DirectionalLight>>, time: Res<Time>) {
    suns.iter_mut()
        .for_each(|mut tf| tf.rotate_y(-time.delta_secs() * PI / 10.0));
}
