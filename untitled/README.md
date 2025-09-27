# Combat Sandbox - Phase 1

A twin-stick combat sandbox built with Bevy and Rapier2D, implementing the foundational systems from the Phase 1 technical specification.

## Current Features

### Core Systems
- **Entity-Team Framework**: All entities have team affiliations (Player, Enemy, Neutral)
- **Team-Based Combat**: Bullets check team membership before applying damage
- **Component Architecture**: Health, velocity, teams as reusable components
- **Physics Integration**: Circle colliders for entities, rectangle for obstacles

### Player Controls
- **Movement**: WASD or Arrow Keys
- **Shooting**: Left Mouse Button or Spacebar
- **Aiming**: Mouse cursor (twin-stick style)
- **Dash**: Left Shift (1-second cooldown, 0.2s invincibility frames)
- **Rate Limiting**: Shooting cooldown prevents spam (10 shots/sec max)

### Enemy AI
- **Multiple Archetypes**: 
  - **Chasers** (Red): Melee enemies that chase the player directly
  - **Shooters** (Purple): Ranged enemies that maintain distance and fire single bullets
  - **Shotgun** (Orange): Mid-range enemies with spread-fire pattern
- **Intelligent Positioning**: Ranged enemies maintain optimal distance
- **Team Awareness**: All enemies target different teams appropriately
- **Health System**: Enemies die when health reaches zero

### World Layout
- **Tactical Arena**: 800x600 room with walls
- **Cover Elements**: Strategic pillars for positioning
- **Collision Physics**: Proper collision response for all entities

## How to Play

1. **Run the game**: `cargo run`
2. **Move**: Use WASD or arrow keys to move your green circle (player)
3. **Aim**: Point your mouse where you want to shoot
4. **Shoot**: Hold left mouse button or spacebar to fire yellow bullets
5. **Dash**: Press Left Shift to dash in your movement direction (grants brief invincibility)
6. **Survive**: Defeat different colored enemies with varied behaviors:
   - **Red** (Chasers): Fast melee attackers
   - **Purple** (Shooters): Ranged attackers that keep distance
   - **Orange** (Shotgun): Spread-fire attackers at medium range
7. **Use Cover**: Hide behind gray pillars to avoid enemy projectiles

## Technical Implementation

### Architecture Highlights
- Built on Bevy ECS (Entity Component System)
- Rapier2D physics engine for realistic movement and collisions
- 60 FPS target with optimized update loops
- Extensible design supporting future features

### Performance
- Handles 20+ entities and 50+ projectiles smoothly
- Delta-time movement for frame-rate independence
- Efficient collision detection with spatial partitioning

## Development Status

✅ **Phase 1 Complete**:
- Core entity system with teams
- Player movement and shooting
- Basic enemy AI
- Team-based damage system
- Collision detection
- Static world layout with obstacles

🚧 **Next Steps (Phase 2)**:
- Multiple enemy archetypes (melee, ranged, shotgun)
- Player dash ability with invincibility frames
- Primary/secondary weapons
- Enhanced AI behaviors
- Visual polish and game juice

## Architecture Notes

This implementation follows the design principles from the Phase 1 specification:
- Single entity collection for all game objects
- Team relationship matrix for damage calculations
- Component-based design for easy extension
- Performance-first approach with 60 FPS stability

The codebase is structured to support the full vision outlined in the game design document while maintaining the MMVP (Minimal Meaningful Viable Product) approach.
