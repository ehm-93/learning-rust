# Iteration 0 - Technical Foundation

## End State Goal
Establish basic Bevy project structure and prove we can build a simple, navigable 3D environment.

## Minimal Features

### Project Setup
- **Bevy Application**: Basic main.rs that launches successfully
- **Basic Plugins**: Default plugins + basic camera controller
- **Third-Party Plugins**:
  - `bevy_rapier3d` - Physics engine for collision and movement
  - `bevy_egui` - Immediate mode GUI for menus
  - `bevy-inspector-egui` - Runtime entity/component inspector (debug only)
  - `bevy_screen_diagnostics` - On-screen FPS and performance metrics (debug only)
  - `bevy_oddio` - Audio engine for sound effects
  - `bevy_tweening` - Animation and interpolation system
  - `rusqlite` - SQLite database for save/load persistence
- **Plugin Configuration**: Proper initialization and setup for all third-party plugins
- **Asset Loading**: Can load and display a simple 3D model
- **Input Handling**: WASD movement, mouse look

### Simple 3D Scene
- **Ground Plane**: Basic floor to walk on
- **Player Controller**: First-person camera that moves smoothly
- **One Static Object**: Cube or simple mesh to interact with
- **Basic Lighting**: Ambient + directional light
- **Flashlight**: Toggleable spotlight attached to camera (F key to toggle, with click sound effect)

### Core Systems Skeleton  
- **Component Structure**: Player, Transform, basic component setup
- **Basic Physics**: Ground collision so player doesn't fall through floor
- **Resource Management**: Simple resource (like Health) that can be modified

### UI Foundation
- **Main Menu**: Start screen with "New Game", "Load Game", "Exit" buttons
- **Pause Menu**: ESC key opens in-game menu with "Save", "Load", "Main Menu", "Exit"
- **Game State Management**: Main Menu ↔ In-Game ↔ Paused state transitions
- **Basic UI**: Simple buttons that respond to clicks

### Save/Load System
- **Player Position**: Save/restore player location in the scene
- **Simple Resource**: Save/restore one basic value (health, etc.)
- **File I/O**: SQLite database for persistence (easily hackable/inspectable)
- **Menu Integration**: Load button actually loads saved data

## Success Criteria
- Can run game and see main menu first
- Can start new game and enter 3D scene
- Can press ESC to pause and access pause menu
- Can toggle flashlight on/off with F key
- Can save game state from pause menu and load it back
- Can walk around with WASD + mouse after loading
- Code is organized with clear component/system separation
- All third-party plugins are properly initialized and configured

This validates core engine + basic game infrastructure patterns.

---

## Component Architecture

### Core Components

#### Player Components
- **Player**: Marker component for the player entity
- **PlayerCamera**: First-person camera controller (sensitivity, pitch, yaw)
- **PlayerMovement**: Movement controller (speed, velocity)
- **Flashlight**: Player's flashlight (enabled state, intensity, range)

#### Saveable Components
- **Saveable**: Marker for entities/components that should be persisted
- **Health**: Simple resource for testing save/load (current, maximum)

### Resources (Global State)

- **GameState**: State machine (MainMenu, InGame, Paused)
- **SavePath**: File path for save data
- **MouseMotion**: Input state for camera control

### System Organization

#### Startup Systems
- `setup_camera` - Spawn player camera entity with components
- `setup_world` - Create ground plane and static objects
- `setup_lighting` - Configure ambient and directional lights
- `setup_ui` - Initialize main menu UI

#### Update Systems (Run in InGame state)
- `player_movement` - Handle WASD input, update velocity
- `apply_movement` - Apply velocity to transform
- `camera_look` - Process mouse movement, update camera rotation
- `toggle_flashlight` - Handle F key, enable/disable spotlight
- `update_flashlight_transform` - Keep flashlight aligned with camera

#### UI Systems
- `main_menu_interaction` - Handle main menu button clicks
- `pause_menu_interaction` - Handle pause menu button clicks
- `handle_pause_input` - ESC key toggles pause state

#### Persistence Systems
- `save_game` - Serialize player position and Health to file
- `load_game` - Deserialize and restore game state

### Extensibility Patterns

#### Component Composition
- Use small, focused components that can be mixed
- Avoid monolithic "PlayerController" - compose from smaller parts
- Easy to add new capabilities by adding components

#### System Ordering
- Use explicit system sets for clear execution order
- Separate input → logic → rendering phases
- Makes adding new systems predictable

#### State-Based Execution
- Systems run only in relevant states
- Clean separation of menu vs gameplay logic
- Easy to add new states later (Loading, GameOver, etc.)

#### Serialization Ready
- `Saveable` marker pattern allows selective persistence
- Component-level `Serialize`/`Deserialize` from start
- Foundation for complex save systems later

### Design Principles

1. **ECS Purity**: Favor components + systems over inheritance
2. **Single Responsibility**: Each system does one thing well
3. **Data-Driven**: Use components for data, systems for logic
4. **Explicit Dependencies**: Clear system ordering, no hidden state
5. **Type Safety**: Leverage Rust's type system for correctness
6. **Modularity**: Each module can be understood independently
