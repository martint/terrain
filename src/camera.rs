use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};
use bevy::input::mouse::MouseButton;

#[derive(Component)]
pub struct CameraController {
    pub move_speed: f32,
    pub look_speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            move_speed: 100.0,
            look_speed: 0.002,
        }
    }
}

pub fn toggle_cursor(
    mut cursor_options: Single<&mut CursorOptions>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    // ESC to release mouse capture
    if keyboard_input.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    
    // Left mouse click to capture mouse
    if mouse_input.just_pressed(MouseButton::Left) && cursor_options.grab_mode == CursorGrabMode::None {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

pub fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut CameraController)>,
    time: Res<Time>,
    cursor_options: Single<&CursorOptions>,
) {
    let Ok((mut transform, controller)) = query.single_mut() else {
        return;
    };

    // Only process mouse movement if cursor is grabbed
    if cursor_options.grab_mode != CursorGrabMode::None {
        for event in mouse_motion_events.read() {
            // Apply yaw rotation (left/right) around the global Y axis
            let yaw_rotation =
                Quat::from_axis_angle(Vec3::Y, -event.delta.x * controller.look_speed);

            // Apply pitch rotation (up/down) around the camera's local right axis
            let right = transform.right();
            let pitch_rotation =
                Quat::from_axis_angle(*right, -event.delta.y * controller.look_speed);

            // Combine rotations: first pitch (local), then yaw (global)
            transform.rotation = yaw_rotation * pitch_rotation * transform.rotation;
        }
    }

    // Handle keyboard movement
    let mut movement = Vec3::ZERO;

    // Get camera's forward and right vectors
    let forward = transform.forward();
    let right = transform.right();

    if keyboard_input.pressed(KeyCode::KeyW) {
        movement += *forward;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        movement -= *forward;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        movement += *right;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        movement -= *right;
    }

    // Normalize movement to prevent faster diagonal movement
    if movement.length() > 0.0 {
        movement = movement.normalize();
    }

    // Apply movement
    transform.translation += movement * controller.move_speed * time.delta_secs();
}
