use crate::settings::Settings;
use bevy::prelude::*;

#[derive(Component)]
pub struct Block;

#[derive(Component)]
pub struct BlockHighlight;

#[derive(Resource)]
pub struct BlockMaterials {
    pub normal: Handle<StandardMaterial>,
    pub highlighted: Handle<StandardMaterial>,
}

pub struct BlocksPlugin;

impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_block_materials)
            .add_systems(Update, highlight_hovered_block);
    }
}

pub fn setup_block_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create materials for blocks
    let normal_material = materials.add(Color::srgb_u8(124, 144, 255));
    let highlighted_material = materials.add(Color::WHITE);

    commands.insert_resource(BlockMaterials {
        normal: normal_material,
        highlighted: highlighted_material,
    });
}

pub fn generate_chunk(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    block_materials: Res<BlockMaterials>,
    settings: Res<Settings>,
) {
    let block_size = settings.world.block_size;
    let chunk_size = settings.world.chunk_size;
    let grid_offset = 0.02; // Small gap between blocks for grid effect

    for z in 0..chunk_size {
        for x in 0..chunk_size {
            // Create slightly smaller blocks to create visual grid lines
            let visual_size = block_size - grid_offset;

            commands.spawn((
                Block,
                Mesh3d(meshes.add(Cuboid::new(visual_size, visual_size, visual_size))),
                MeshMaterial3d(block_materials.normal.clone()),
                Transform::from_xyz(x as f32 * block_size, 0.0, z as f32 * block_size),
                Visibility::Visible,
            ));
        }
    }
}

pub fn highlight_hovered_block(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    block_materials: Res<BlockMaterials>,
    mut blocks: Query<(Entity, &Transform, &mut MeshMaterial3d<StandardMaterial>), With<Block>>,
    highlighted: Query<Entity, With<BlockHighlight>>,
) {
    // Remove previous highlight
    for entity in highlighted.iter() {
        if let Ok((_, _, mut material)) = blocks.get_mut(entity) {
            material.0 = block_materials.normal.clone();
        }
        commands.entity(entity).remove::<BlockHighlight>();
    }

    // Get the cursor position and cast ray
    let (camera, camera_transform) = match camera_query.get_single() {
        Ok(result) => result,
        Err(_) => return,
    };

    let window = match windows.get_single() {
        Ok(window) => window,
        Err(_) => return,
    };

    // Get cursor position in the center of the screen (since cursor is locked)
    let cursor_position = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    // Cast ray from cursor position
    let ray = match camera.viewport_to_world(camera_transform, cursor_position) {
        Ok(ray) => ray,
        Err(_) => return,
    };

    // Find the closest block hit by the ray
    let max_distance = 5.0; // Maximum distance for block selection
    let ray_direction = ray.direction.normalize();

    let mut closest_block = None;
    let mut closest_distance = f32::MAX;

    for (entity, transform, _) in blocks.iter() {
        let block_pos = transform.translation;
        let block_size = 1.0; // Using standard block size

        // Simple AABB ray intersection test
        let min = block_pos - Vec3::splat(block_size / 2.0);
        let max = block_pos + Vec3::splat(block_size / 2.0);

        // Ray-AABB intersection algorithm
        let t1 = (min.x - ray.origin.x) / ray_direction.x;
        let t2 = (max.x - ray.origin.x) / ray_direction.x;
        let t3 = (min.y - ray.origin.y) / ray_direction.y;
        let t4 = (max.y - ray.origin.y) / ray_direction.y;
        let t5 = (min.z - ray.origin.z) / ray_direction.z;
        let t6 = (max.z - ray.origin.z) / ray_direction.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        // If tmax < 0, ray is intersecting AABB, but entire AABB is behind ray
        if tmax < 0.0 {
            continue;
        }

        // If tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            continue;
        }

        // Ray intersects, check if it's the closest
        if tmin > 0.0 && tmin < max_distance && tmin < closest_distance {
            closest_distance = tmin;
            closest_block = Some(entity);
        }
    }

    // Highlight the closest block
    if let Some(entity) = closest_block {
        if let Ok((_, _, mut material)) = blocks.get_mut(entity) {
            material.0 = block_materials.highlighted.clone();
        }
        commands.entity(entity).insert(BlockHighlight);
    }
}
