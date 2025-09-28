# Bevy-Rapier Shadowcasting Demo

A Bevy game demonstrating 2D flashlight-style shadowcasting with Rapier physics integration.

## Features

- **Moveable Player**: A green circle (pip) that you can control with WASD keys
- **Flashlight Effect**: Dark environment with a bright flashlight beam following your mouse
- **Real Shadow Casting**: Obstacles cast realistic shadows, creating dark areas behind them
- **135-Degree Vision Arc**: Wide flashlight beam for better shadow demonstration
- **Obstacle Detection**: Gray rectangular and circular obstacles that block the light
- **Physics Integration**: Uses Bevy-Rapier for precise collision detection and raycasting
- **Smooth Shadows**: 128 rays create smooth shadow edges

## Controls

- **WASD**: Move the player around
- **Mouse**: The player's vision cone follows your mouse cursor
- **Esc**: Close the application

## How It Works

1. The player acts as a flashlight source in a dark environment
2. 128 rays are cast in a 135-degree arc toward the mouse direction
3. Each ray uses Rapier's physics engine to detect collisions with obstacles
4. Areas where rays are blocked create realistic shadows behind obstacles
5. The result is a warm, bright flashlight beam that clearly shows shadow casting

## Technical Details

- **Engine**: Bevy 0.16
- **Physics**: Bevy-Rapier 0.31
- **Rendering**: Custom mesh generation for the flashlight beam
- **Raycasting**: 128 rays cast in a 135-degree arc for smooth shadow edges
- **Visual Style**: Dark background with warm flashlight colors for dramatic effect

## Running

```bash
cargo run
```

## Code Structure

- `main.rs`: Complete demo in a single file
- Player movement system
- Mouse tracking system  
- Shadowcast visual generation system
- Physics world setup with scattered obstacles

## Future Improvements

- Multiple flashlight sources
- Dynamic/moving obstacles
- Light intensity falloff with distance
- Colored lighting effects
- Performance optimizations for even more rays
- Soft shadow edges

Enjoy experimenting with the shadowcasting demo!
