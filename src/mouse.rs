use bevy::prelude::*;

#[derive(Resource, Default, Copy, Clone, PartialEq)]
pub struct MouseCoords(pub(crate) Vec2);

pub struct MousePlugin;

impl Plugin for MousePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MouseCoords>()
            .add_systems(PreUpdate, cursor_system);
    }
}

fn cursor_system(
    mut coords: ResMut<MouseCoords>,
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
) {
    let Ok((camera, camera_transform)) = camera.get_single() else {
        return;
    };
    
    let window = window.single();

    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_position;
    }
}
