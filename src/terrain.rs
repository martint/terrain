use bevy::asset::RenderAssetUsages;
use bevy::pbr::wireframe::Wireframe;
use bevy::prelude::*;
use bevy_mesh::Indices;
use wgpu_types::PrimitiveTopology;

#[derive(Component, Clone, Copy)]
pub struct Tile {}

#[derive(Component)]
pub struct NormalLines;

#[derive(Resource)]
pub struct TerrainManager {
    pub wireframe_mode: bool,
    pub show_normals: bool,
}

impl Default for TerrainManager {
    fn default() -> Self {
        Self {
            wireframe_mode: false,
            show_normals: false,
        }
    }
}

pub fn toggle_wireframe_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut terrain_manager: ResMut<TerrainManager>,
    mut commands: Commands,
    tile_query: Query<(Entity, &MeshMaterial3d<StandardMaterial>), With<Tile>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        terrain_manager.wireframe_mode = !terrain_manager.wireframe_mode;

        for (entity, _tile) in tile_query.iter() {
            if terrain_manager.wireframe_mode {
                commands.entity(entity).insert(Wireframe);
            } else {
                commands.entity(entity).remove::<Wireframe>();
            }
        }
    }
}

pub fn toggle_normals_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut terrain_manager: ResMut<TerrainManager>,
    mut normal_lines_query: Query<&mut Visibility, With<NormalLines>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyN) {
        terrain_manager.show_normals = !terrain_manager.show_normals;

        if let Ok(mut visibility) = normal_lines_query.single_mut() {
            *visibility = if terrain_manager.show_normals {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// Initialize the terrain system with 6 cube faces
pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_manager: Res<TerrainManager>,
) {
    let resolution = 5000;
    let vertex_count = (resolution + 1) * (resolution + 1);

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);

    for row in 0..=resolution {
        for col in 0..=resolution {
            let x = row as f32 - resolution as f32 / 2.0;
            // let x = (row as f32);
            let z = col as f32 - resolution as f32 / 2.0;
            // let z = (col as f32);
            let (y, normal) = sample(x, z);
            positions.push([x, y, z]);
            normals.push([normal.x, normal.y, normal.z]);
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity(resolution * resolution * 6);
    for row in 0..resolution {
        for col in 0..resolution {
            let top_left = (row * (resolution + 1) + col) as u32;
            let top_right = top_left + 1;
            let bottom_left = ((row + 1) * (resolution + 1) + col) as u32;
            let bottom_right = bottom_left + 1;

            // Two triangles per quad - clockwise winding for outward-facing triangles
            indices.push(top_left);
            indices.push(top_right);
            indices.push(bottom_left);

            indices.push(top_right);
            indices.push(bottom_right);
            indices.push(bottom_left);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
    mesh.insert_indices(Indices::U32(indices));

    let mut tile = commands.spawn((
        Tile {},
        MeshMaterial3d(materials.add(Color::srgb_u8(228, 172, 155))),
        Mesh3d(meshes.add(mesh)),
    ));

    if terrain_manager.wireframe_mode {
        tile.insert(Wireframe);
    }

    spawn_normals(&mut commands, &mut meshes, &mut materials, &positions, &normals);
}

fn spawn_normals(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    positions: &Vec<[f32; 3]>,
    normals: &Vec<[f32; 3]>,
) {
    // Create normal visualization mesh
    let normal_length = 1.0;
    let mut line_positions: Vec<[f32; 3]> = Vec::new();

    for (pos, normal) in positions.iter().zip(normals.iter()) {
        line_positions.push(*pos);
        line_positions.push([
            pos[0] + normal[0] * normal_length,
            pos[1] + normal[1] * normal_length,
            pos[2] + normal[2] * normal_length,
        ]);
    }

    commands.spawn((
        NormalLines,
        Mesh3d(
            meshes.add(
                Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
                    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line_positions.clone()),
            ),
        ),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 1.0),
            unlit: true,
            ..default()
        })),
        Visibility::Hidden, // Start hidden
    ));
}

fn sample(x: f32, z: f32) -> (f32, Vec3) {
    let amplitude = 300.0;
    let scale = 800.0;

    let (y, d) = fbm(Vec2::new(x, z) / scale);

    let adjusted_y = y * amplitude;
    let adjusted_d = d * amplitude / scale;

    (
        adjusted_y,
        Vec3::new(-adjusted_d.x, 1.0, -adjusted_d.y).normalize(),
    )
}

fn hash(p: Vec2) -> f32 {
    // Convert to signed integers (floored)
    let ix = p.x.floor() as i32;
    let iy = p.y.floor() as i32;

    let seed = 3266489917_u32;
    let mut h = seed
        .wrapping_add((ix as u32).wrapping_mul(374761393))
        .wrapping_add((iy as u32).wrapping_mul(668265263));

    // Avalanche step (bit diffusion)
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);

    // Convert to float in [0, 1)
    (h as f32) * (1.0 / 4294967296.0)
}

fn smoothstep(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn noise(t: Vec2) -> (f32, Vec2) // value, dx, dy
{
    let p = t.floor();

    let a = hash(p + Vec2::new(0.0, 0.0));
    let b = hash(p + Vec2::new(1.0, 0.0));
    let c = hash(p + Vec2::new(0.0, 1.0));
    let d = hash(p + Vec2::new(1.0, 1.0));

    let k0 = a;
    let k1 = b - a;
    let k2 = c - a;
    let k3 = a - b - c + d;

    let w = t.fract_gl();
    let (sx, sz) = (smoothstep(w.x), smoothstep(w.y));

    let value = k0 + k1 * sx + k2 * sz + k3 * sx * sz;

    let ds = 6.0 * w * (1.0 - w);
    let dx = ds.x * (k1 + k3 * sz);
    let dy = ds.y * (k2 + k3 * sx);

    (value, Vec2::new(dx, dy))
}

const ROTATION: Mat2 = Mat2::from_cols_array(&[0.8, 0.6, -0.6, 0.8]);
const ROTATION_TRANSPOSE: Mat2 = Mat2::from_cols_array(&[0.8, -0.6, 0.6, 0.8]);

fn fbm(point: Vec2) -> (f32, Vec2) // value, dx, dy
{
    let scale_factor = 2.0;

    let mut p = point;
    let mut scale = 1.0;

    let mut rotation = Mat2::IDENTITY;

    let mut value = 0.0;
    let mut derivative = Vec2::new(0.0, 0.0);

    for _ in 0..11 {
        let (noise, noise_derivative) = noise(p);

        value += scale * noise;
        derivative += scale * rotation * noise_derivative;

        scale /= scale_factor;

        p = scale_factor * ROTATION * p;
        rotation = scale_factor * ROTATION_TRANSPOSE * rotation;
    }

    (value, derivative)
}