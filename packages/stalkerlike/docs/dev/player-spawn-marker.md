# Player Spawn Marker

## Overview
The editor now includes a **Player Spawn** marker that allows level designers to specify where the player should spawn when entering a scene in game mode.

## Usage

### In the Editor
1. Open the editor with `cargo run -- --editor`
2. In the left-side asset browser panel, scroll down and select **"Player Spawn"**
3. Click in the viewport to place the marker at the desired spawn location
4. The marker appears as a **bright green cone** pointing upward for easy visibility
5. Press `ESC` to exit placement mode
6. Save the scene (Ctrl+S) - the marker will be saved with the scene

### Visual Appearance
- **Shape**: Upward-pointing cone (arrow)
- **Color**: Bright green (RGB: 0.2, 1.0, 0.2)
- **Default Size**: 0.5m wide × 2m tall

### Technical Details

#### Component
The player spawn marker has a `PlayerSpawn` component attached to it. Game mode code can query for this component to find the spawn position:

```rust
// Example query to find player spawn point
fn find_player_spawn(
    spawn_query: Query<&Transform, With<PlayerSpawn>>
) {
    if let Ok(spawn_transform) = spawn_query.single() {
        let spawn_position = spawn_transform.translation;
        // Spawn player at spawn_position
    }
}
```

#### Serialization
The player spawn marker properly serializes to YAML with:
- Transform data (position, rotation, scale)
- Mesh type (PlayerSpawn)
- Material color (bright green)
- PlayerSpawn component marker

Example YAML:
```yaml
entities:
  - name: Player Spawn
    transform:
      position: [5.0, 0.0, 3.0]
      rotation: [0.0, 0.0, 0.0, 1.0]
      scale: [0.5, 2.0, 0.5]
    components:
      - type: Mesh
        primitive_type: PlayerSpawn
      - type: Material
        base_color: [0.2, 1.0, 0.2, 1.0]
      - type: PlayerSpawn
```

#### Implementation Files
- **Primitive Type**: `src/editor/objects/primitives.rs` - Added `PlayerSpawn` to `PrimitiveType` enum
- **Component**: `src/editor/core/types.rs` - Added `PlayerSpawn` component marker
- **Placement**: `src/editor/objects/placement.rs` - Automatically adds `PlayerSpawn` component when placed
- **Serialization**: `src/editor/persistence/scene.rs` - Handles save/load of PlayerSpawn component

## Best Practices
- **One spawn per scene**: While you can place multiple markers, game mode should typically only use one
- **Clear positioning**: Place the marker on solid ground at a safe, unobstructed location
- **Rotation matters**: If using the marker's rotation, orient it to face the desired starting direction
- **Name it**: The marker is automatically named "Player Spawn" for easy identification in the hierarchy

## Future Enhancements
- Support for multiple spawn points with priorities
- Named spawn points for different entry scenarios (e.g., "main_entrance", "emergency_exit")
- Spawn radius for randomized starting positions
- Preview mode to test spawn point visibility and safety
