# MMVP Technical Specification - Combat Sandbox

## Vision
Build the minimal combat experience that validates core gameplay: "Is twin-stick shooting with varied enemies and tactical positioning fun?" Everything else is scope creep until this question is answered.

## Core Architecture

### Entity-Team Framework
All game objects are entities with team affiliations, creating a unified system that scales naturally:

```
Entity System:
├── Team-based collision (bullets respect team membership)
├── Unified behavior system (AI targets by team, not hardcoded types)  
├── Scalable relationships (neutral NPCs, allies, team switching)
└── Future-proof for complex faction dynamics
```

### Game State Structure
- **Single entity collection** - all players, enemies, bullets in one list
- **Static obstacle array** - pillars and walls for tactical positioning
- **Team relationship matrix** - which teams can damage which teams
- **Minimal state tracking** - focus on core combat loop

## Technical Systems

### Entity Management
- **Component-based entities** with position, velocity, health, team
- **Entity factory system** for spawning different archetypes
- **Efficient update loops** processing all entities uniformly
- **Memory-conscious design** - entity pooling for bullets

### Combat System
- **Weapon diversity** - primary/secondary attacks with different characteristics
- **Projectile variety** - different speeds, sizes, patterns, damage
- **Team-aware damage** - bullets check team before applying damage
- **Player abilities** - dash with invincibility frames and cooldowns

### AI Behaviors
- **Archetype-driven** - melee chasers, ranged snipers, rapid-fire, shotgun spread
- **Pathfinding basics** - navigate around obstacles toward targets
- **Line-of-sight** calculations for ranged engagement
- **State machines** - idle, chase, attack, cooldown behaviors

### Physics & Movement
- **Fixed timestep** game loop with delta time interpolation
- **Circle collision** for entities, circle-rectangle for obstacles
- **Velocity-based movement** with collision response
- **Spatial awareness** - entities react to nearby teammates and threats

### World Representation
- **Static room layout** - rectangular bounds with pillar obstacles
- **Tactical positioning** - cover mechanics and sight lines
- **Collision boundaries** - walls stop entities and projectiles
- **Clear spatial rules** - predictable physics for tactical gameplay

## Performance Targets

### Framerate & Responsiveness
- **60 FPS stable** with 20+ entities and 50+ projectiles
- **Sub-16ms frame times** for responsive twin-stick controls
- **Smooth movement** with proper delta-time scaling
- **Immediate input response** - no noticeable input lag

### Scalability Considerations
- **Efficient collision detection** - spatial partitioning if needed
- **Bullet pooling** to avoid allocation churn
- **Batch processing** where possible
- **Profiling hooks** for performance monitoring

## Technology Stack

### Core Framework
- **Rust + macroquad** for rapid prototyping and cross-platform support
- **Standard library only** - minimal external dependencies
- **Component-based architecture** - easy to extend and modify
- **Data-driven design** where practical

### Development Approach
- **Iterative prototyping** - get basic loop working, then add complexity
- **Immediate feedback** - visual and audio cues for all interactions
- **Playtesting focus** - optimize for fun, not technical perfection
- **Extensible foundation** - architecture supports future expansion

## Success Metrics

### Technical Validation
- Stable 60 FPS with target entity counts
- Responsive controls with no input lag
- Collision detection accuracy and performance
- Clean, extensible codebase architecture

### Gameplay Validation
- **Core loop engagement** - "Do I want to play again?"
- **Combat variety** - Different enemies create different challenges
- **Tactical depth** - Cover and positioning matter
- **Skill progression** - Players improve through practice

## Development Phases

### Phase 1: Foundation
- Basic entity system with teams
- Player movement and shooting
- Simple enemy spawning and basic AI
- Collision detection working

### Phase 2: Combat Variety
- Multiple enemy archetypes with different behaviors
- Primary/secondary weapons with distinct feel
- Dash ability with proper game feel
- Basic obstacle placement

### Phase 3: Polish & Validation
- Juice and feedback - screen shake, particles, sound
- Balance tuning for engaging combat
- Performance optimization
- Playtesting and iteration

## Future Architecture Considerations

### Planned Extensibility
- **Procedural generation** hooks in world representation
- **Loot/inventory** systems can attach to entities
- **Progression systems** can modify entity capabilities
- **Networking** foundation compatible with entity-team model

### Technical Debt Management
- Keep systems simple until complexity is justified
- Document assumptions and design decisions
- Profile early and often
- Plan refactoring points before they become necessary

---

**Core Philosophy:** Build the minimum system that proves the fun, with architecture that won't fight future expansion. Every technical decision serves the goal of validating core gameplay as quickly as possible.
