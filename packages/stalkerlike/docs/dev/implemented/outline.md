# Outline System Dev Spec

## Components

**`Outlined`** - marker component, add to entities to outline them

**`OutlineMarker`** - internal component tracking outline child entities
- Stores parent entity reference

## Resources

**`OutlineMaterial`** - unlit material handle, created at startup via FromWorld
- Color: bright yellow (1.0, 0.95, 0.0)
- Alpha mode: Opaque
- Unlit: true
- Cull mode: Front (culls front faces, shows only back faces)

## Systems

### 1. Spawn Outlines
Runs when `Outlined` added

```
For each entity with Outlined (added):
  - Get original mesh from Assets<Mesh>
  - Create outline mesh with inverted normals
  - Spawn child entity with:
    - Inverted mesh
    - OutlineMaterial (front-face culled)
    - Scale: 1.05 (5% larger)
    - OutlineMarker component (tracks parent)
```

### 2. Despawn Outlines
Runs when `Outlined` removed

```
For each entity with Outlined (removed):
  - Find children with OutlineMarker
  - Despawn them (and their mesh assets)
```

### 3. Sync Transforms
Runs every frame

```
For each entity with OutlineMarker:
  - If parent still has Outlined:
    - Zero out local position/rotation (child of parent)
    - Maintain 1.05x scale ratio
```

## Rendering Technique

**Inverted Hull Method**:
1. Clone parent mesh and invert all vertex normals (flip direction)
2. Apply front-face culling to outline material
3. Scale mesh 5% larger than parent
4. Result: Only the "back faces" of the scaled mesh are visible, creating a halo effect around the silhouette

This approach ensures outlines only appear where they extend beyond the parent object's edges, never on top of the parent surface.

## Edge Cases

- Parent despawned → child auto-despawns (Bevy hierarchy)
- Multiple outlined entities → each gets independent outline
- Parent scale changes → outline scales proportionally via sync system
- Mesh without normals → outline won't render correctly (requires ATTRIBUTE_NORMAL)

## Performance

- Adds N draw calls for N outlined entities
- Creates N outline meshes in Assets<Mesh> (clones with inverted normals)
- Negligible for typical selection counts (<100)
- Outline meshes are cleaned up when parent is despawned

## Implementation Details

Located in `src/editor/objects/outline.rs`:
- `create_outline_mesh()` - helper to invert mesh normals
- `spawn_outlines()` - system with Added<Outlined> filter
- `despawn_outlines()` - system with RemovedComponents<Outlined>
- `sync_outline_transforms()` - system running every frame

Integrated in `EditorPlugin` via selection systems.

