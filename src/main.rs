use bevy::camera::Exposure;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::light::light_consts::lux;
use bevy::light::AtmosphereEnvironmentMapLight;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::Atmosphere;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::window::{CursorGrabMode, CursorOptions};
use std::f32::consts::PI;

mod camera;
mod camera_widget;
mod terrain;

use crate::terrain::TerrainManager;
use camera_widget::{setup_camera_widget, CameraWidgetPlugin, MainCamera};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum Stage {
    #[default]
    Loading,
    Running,
}

#[derive(Component)]
struct LoadingScreen;

#[derive(Component)]
struct LoadingText;

#[derive(Component)]
struct LoadingSpinner;

#[derive(Component)]
struct LoadingDot(usize);

#[derive(Component)]
struct TerrainGenerationTask(Task<(Mesh, Vec<[f32; 3]>, Vec<[f32; 3]>)>);

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
        .init_resource::<TerrainManager>()
        .init_state::<Stage>()
        .add_systems(Startup, setup_loading_screen)
        .add_systems(
            Update,
            (
                animate_loading_screen,
                start_game,
                start_terrain_generation,
                terrain::check_terrain_generation,
            )
                .run_if(in_state(Stage::Loading)),
        )
        .add_systems(
            OnEnter(Stage::Running),
            (
                setup_environment,
                setup_ui,
                setup_cursor,
                setup_camera_widget,
                cleanup_loading_screen,
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
                dynamic_scene,
            )
                .run_if(in_state(Stage::Running)),
        )
        .run();
}

fn setup_loading_screen(mut commands: Commands) {
    commands.spawn((Camera2d, LoadingScreen));

    // Full screen overlay
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.15)),
            LoadingScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Loading Terrain..."),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                LoadingText,
            ));

            // Spinner
            parent
                .spawn((
                    Node {
                        margin: UiRect::top(Val::Px(40.0)),
                        column_gap: Val::Px(10.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    LoadingSpinner,
                ))
                .with_children(|dots_parent| {
                    for i in 0..3 {
                        dots_parent.spawn((
                            Node {
                                width: Val::Px(12.0),
                                height: Val::Px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.3, 0.6, 0.9)),
                            BorderRadius::all(Val::Px(6.0)),
                            LoadingDot(i),
                        ));
                    }
                });
        });
}

fn start_game(mut next_state: ResMut<NextState<Stage>>, terrain_manager: Res<TerrainManager>) {
    if terrain_manager.loaded {
        next_state.set(Stage::Running);
    }
}

fn start_terrain_generation(mut commands: Commands, task_query: Query<&TerrainGenerationTask>) {
    if task_query.is_empty() {
        let thread_pool = AsyncComputeTaskPool::get();

        let task = thread_pool.spawn(async move { terrain::generate_terrain_mesh() });

        commands.spawn(TerrainGenerationTask(task));
    }
}

fn animate_loading_screen(
    time: Res<Time>,
    mut dot_query: Query<(&mut BackgroundColor, &LoadingDot)>,
) {
    for (mut bg_color, dot) in dot_query.iter_mut() {
        let offset = dot.0 as f32 * 0.3; // Stagger the animation
        let scale = ((time.elapsed_secs() * 3.0 + offset).sin() * 0.5 + 0.5).clamp(0.3, 1.0);
        bg_color.0 = Color::srgb(0.3 * scale, 0.6 * scale, 0.9 * scale);
    }
}

fn cleanup_loading_screen(
    mut commands: Commands,
    loading_screen_query: Query<Entity, With<LoadingScreen>>,
) {
    for entity in loading_screen_query.iter() {
        commands.entity(entity).despawn();
    }
}

fn setup_environment(mut commands: Commands) {
    let position = Vec3::new(71.0, 406.0, 1008.0);
    let pitch = -10.0_f32.to_radians();
    let heading = 335.0_f32.to_radians();
    commands.spawn((
        Camera3d::default(),
        Atmosphere::EARTH,
        Exposure::SUNLIGHT,
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
        Tonemapping::AcesFitted,
        camera::CameraController::default(),
        Transform::from_translation(position).with_rotation(Quat::from_rotation_y(-heading) * Quat::from_rotation_x(pitch)),
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
