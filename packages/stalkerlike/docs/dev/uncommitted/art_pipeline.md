# Art Pipeline

## Overview
Low-poly aesthetic inspired by Deep Rock Galactic and Metro 2033, emphasizing oppressive underground atmosphere through deliberate artistic constraints.

## Visual Style Guide

### Core Aesthetic
- **Polygon Budget**: 100-500 for props, 500-2000 for characters, 200-1000 for tunnel modules
- **Shading**: Flat shading (hard edges) for industrial feel
- **Texturing**: Primarily vertex colors, minimal texture maps
- **Lighting**: Sparse, harsh, creating deep shadows

### Color Palette
```yaml
Active Areas:
  metal: [0.3, 0.3, 0.35]        # Dark gray-blue
  rust: [0.5, 0.3, 0.2]           # Muted orange-brown
  emergency: [0.8, 0.2, 0.1]      # Deep red
  work_light: [1.0, 0.9, 0.7]     # Warm yellow

Abandoned Zones:
  frost: [0.7, 0.75, 0.8]         # Light blue-gray
  dead_metal: [0.2, 0.2, 0.22]   # Nearly black
  vacuum_ice: [0.9, 0.92, 0.95]  # Almost white

C-7 Infected:
  crystal: [0.2, 0.8, 0.7]        # Sick teal
  corruption: [0.4, 0.5, 0.3]     # Yellow-green
  glow: [0.5, 1.0, 0.9]          # Bright cyan emission
```

## Asset Creation Pipeline

### Blender Workflow

#### 1. Modeling
```python
# Blender best practices for game assets
- Start with primitive shapes
- Keep topology clean (quads preferred)
- Delete unseen faces aggressively
- Apply all transforms before export
- Keep pivot at logical position (door hinges, etc)
```

#### 2. Vertex Coloring
```
Vertex Paint Mode:
1. Select all faces → Shade Flat
2. Switch to Vertex Paint mode
3. Paint base color
4. Add wear/dirt in crevices (darker colors)
5. Add edge highlights sparingly
```

#### 3. UV Mapping (When Needed)
```
For texture atlases only:
1. Create 256x256 or 512x512 palette texture
2. UV unwrap to specific color squares
3. Share single texture across many objects
```

#### 4. Export Settings
```
glTF 2.0 Binary (.glb):
- Format: glTF Binary
- Transform: +Y Up
- Geometry:
  ✓ Apply Modifiers
  ✓ Vertex Colors
  ✓ Tangents (if using normal maps)
- Materials: Export (for vertex colors)
```

## Modular Asset System

### Tunnel Modules
```yaml
Core Set (5 pieces minimum):
  tunnel_straight:
    size: [4, 3, 4]  # meters
    connections: [north, south]
    variants: [clean, damaged, sealed]
    
  tunnel_corner:
    size: [4, 3, 4]
    connections: [north, east]
    
  tunnel_T:
    size: [4, 3, 4]
    connections: [north, south, east]
    
  tunnel_cross:
    size: [4, 3, 4]
    connections: [north, south, east, west]
    
  tunnel_end:
    size: [4, 3, 4]
    connections: [north]
```

### Environmental Storytelling Props
```yaml
Industrial:
  - mining_drill (200 polys)
  - ore_cart (150 polys)
  - pressure_valve (80 polys)
  - cable_spool (100 polys)
  
Abandoned:
  - sleeping_bag (50 polys)
  - makeshift_stove (120 polys)
  - personal_effects (30-50 polys each)
  - corpse_miner (500 polys)
  
C-7 Infected:
  - crystal_growth_small (50 polys)
  - crystal_growth_large (200 polys)
  - dissolved_wall (modified tunnel piece)
  - impossible_geometry (non-euclidean mesh)
```

## Optimization Techniques

### LOD Strategy
```
LOD0 (0-20m): Full detail
LOD1 (20-50m): Remove bolts/rivets
LOD2 (50-100m): Simplified geometry
LOD3 (100m+): Box with material
```

### Batching
- Use same material for multiple objects
- Combine static meshes in Blender
- Instance repeated elements (pipes, lights)

### Texture Atlasing
```
Single 1024x1024 atlas contains:
- 16x16 grid of color swatches
- Decal sheet (signs, markings)
- Emission masks for lights
```

## Depth-Based Visual Language

### Surface (0 to -100m)
- Clean geometry, maintained
- Proper lighting fixtures
- Corporate signage
- Minimal rust/wear

### Active Mining (-100 to -1000m)
- Industrial wear visible
- Mixed lighting (work lights + emergency)
- Equipment in various states
- Safety markings prominent

### Frontier (-1000 to -5000m)
- Makeshift repairs visible
- Failing infrastructure
- Personal modifications
- Faction graffiti

### The Deep (-5000m+)
- Geometry distortion
- C-7 crystal growth
- Impossible spaces
- Bio-luminescent elements

## Quick Asset Creation

### 10-Minute Tunnel Module
1. Add Cube → Scale to tunnel shape
2. Inset top face → Delete (hollow)
3. Add edge loops for detail (I → edge loop)
4. Select edges → Bevel (Ctrl+B) for industrial feel
5. Vertex paint base color
6. Add wear with darker colors
7. Export as .glb

### 5-Minute Prop
1. Start with primitive (cube/cylinder)
2. Add minimal detail with extrude (E)
3. Delete hidden faces
4. Apply single vertex color
5. Export

## Material Guidelines

### Vertex Color Materials
```rust
// In Bevy
StandardMaterial {
    base_color: Color::WHITE,  // Multiplies with vertex colors
    metallic: 0.8,              // For metal surfaces
    roughness: 0.4,
    ..default()
}
```

### Special Materials
```yaml
Emergency Light:
  emission: [1.0, 0.2, 0.1]
  intensity: 2.0
  
C-7 Crystal:
  base_color: [0.2, 0.8, 0.7]
  emission: [0.1, 0.4, 0.35]
  alpha_mode: blend
  
Frost:
  base_color: [0.9, 0.92, 0.95]
  roughness: 0.1
  metallic: 0.0
```

## Animation Requirements

### Mechanical
- Doors: Simple rotation/slide
- Fans: Continuous rotation
- Elevators: Linear movement
- Sparks: Particle system

### Environmental
- Flickering lights (code-driven)
- Steam vents (particle system)
- Dripping water (simple mesh + shader)
- C-7 pulsing (emission intensity)

## Performance Budgets

### Per Frame Targets (60 FPS)
```
Geometry: 
  - 50k triangles visible
  - 200 draw calls max
  
Textures:
  - 2-4 texture atlases total
  - Primarily vertex colors
  
Lights:
  - 3-5 shadow-casting
  - 10-20 point lights
```

## Tools and Resources

### Essential Software
- **Blender** (3.6+): Modeling and vertex painting
- **GIMP/Krita**: Texture atlas creation
- **AssetForge**: Quick prototyping (optional)

### Blender Addons
- **Batch Operations**: For managing multiple assets
- **Vertex Color Master**: Advanced vertex painting
- **glTF Exporter**: Built-in, ensure updated

### Reference Resources
- Metro 2033 artbook
- Deep Rock Galactic GDC talks
- Alien: Isolation environment design
- Dead Space industrial reference

## Asset Validation Checklist

Before export:
- [ ] All transforms applied
- [ ] No duplicate vertices
- [ ] Faces culled (backface/hidden)
- [ ] Vertex colors applied
- [ ] Pivot point logical
- [ ] Scale appropriate (1 unit = 1 meter)
- [ ] Named descriptively
- [ ] Collision simplified or separate

## Git Storage

```
assets/
├── models/
│   ├── modules/        # .glb files
│   ├── props/          # .glb files
│   └── sources/        # .blend files (Git LFS)
├── textures/
│   └── atlas.png       # Minimal textures
└── materials/
    └── definitions.ron  # Material definitions
```
