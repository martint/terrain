use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

pub struct CameraWidgetPlugin;

impl Plugin for CameraWidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_widget_axes);
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
struct WidgetAxes;

pub fn setup_camera_widget(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 150,
        height: 150,
        ..default()
    };

    // Create a render target image
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);

    let widget_layer = RenderLayers::layer(1);

    commands.spawn((
        Camera3d::default(),
        Camera {
            order: -1, // render before main camera
            target: bevy::camera::RenderTarget::Image(image_handle.clone().into()),
            clear_color: ClearColorConfig::Custom(Color::srgba(0.15, 0.15, 0.15, 0.5)),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 3.1)).looking_at(Vec3::ZERO, Vec3::Y),
        widget_layer.clone(),
    ));

    // Create UI node to display the texture in the top-right corner
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            right: Val::Px(10.0),
            top: Val::Px(10.0),
            width: Val::Px(100.0),
            height: Val::Px(100.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                ImageNode::new(image_handle),
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        });

    // --- Simple axes meshes with UNLIT materials (bright and self-illuminated)
    let x_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        unlit: true,
        ..default()
    });
    let y_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        unlit: true,
        ..default()
    });
    let z_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        unlit: true,
        ..default()
    });

    let shaft = meshes.add(Cylinder::new(0.05, 1.0));
    let tip = meshes.add(Cone::new(0.1, 0.2));

    let axes = commands
        .spawn((
            WidgetAxes,
            Transform::IDENTITY,
            widget_layer.clone(),
        ))
        .id();

    // X axis (red) - pointing right
    commands.spawn((
        Mesh3d(shaft.clone()),
        MeshMaterial3d(x_material.clone()),
        Transform::from_translation(Vec3::X * 0.5)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        ChildOf(axes),
        widget_layer.clone(),
    ));

    commands.spawn((
        Mesh3d(tip.clone()),
        MeshMaterial3d(x_material),
        Transform::from_translation(Vec3::X * 1.075)
            .with_rotation(Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2)),
        ChildOf(axes),
        widget_layer.clone(),
    ));

    // Y axis (green) - pointing up
    commands.spawn((
        Mesh3d(shaft.clone()),
        MeshMaterial3d(y_material.clone()),
        Transform::from_translation(Vec3::Y * 0.5),
        ChildOf(axes),
        widget_layer.clone(),
    ));

    commands.spawn((
        Mesh3d(tip.clone()),
        MeshMaterial3d(y_material),
        Transform::from_translation(Vec3::Y * 1.075),
        ChildOf(axes),
        widget_layer.clone(),
    ));

    // Z axis (blue) - pointing toward camera
    commands.spawn((
        Mesh3d(shaft.clone()),
        MeshMaterial3d(z_material.clone()),
        Transform::from_translation(Vec3::Z * 0.5)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ChildOf(axes),
        widget_layer.clone(),
    ));

    commands.spawn((
        Mesh3d(tip.clone()),
        MeshMaterial3d(z_material),
        Transform::from_translation(Vec3::Z * 1.075)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ChildOf(axes),
        widget_layer.clone(),
    ));
}

fn sync_widget_axes(
    main_camera_query: Query<&Transform, (With<MainCamera>, Without<WidgetAxes>)>,
    mut axes_query: Query<&mut Transform, (With<WidgetAxes>, Without<MainCamera>)>,
) {
    let Ok(main_camera) = main_camera_query.single() else {
        return;
    };
    let Ok(mut axes) = axes_query.single_mut() else {
        return;
    };

    axes.rotation = main_camera.rotation.inverse();
}
