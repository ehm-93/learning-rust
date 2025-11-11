# Development Documentation

This directory contains technical specifications, iteration plans, and design documents for the Stalkerlike project.

## Directory Structure

### `/` (Root)
- **`iter-00.md`** - Iteration 0: Game mode foundations (player controller, basic 3D scene, save/load system)
- **`iter-01.md`** - Iteration 1: Editor MVP (camera controller, transform gizmos, primitive placement, inspector panel)

### `/implemented/`
Specifications for completed features that have been fully implemented and tested.

- **`outline.md`** - Selection outline system using inverted normals technique

### `/in-progress/`
Active development specifications currently being implemented or refined.

- **`art_pipeline.md`** - Low-poly aesthetic guidelines, vertex colors, poly budget constraints
- **`editor.md`** - Full editor vision and feature roadmap

### `/superceded/`
Archived specifications that have been replaced by newer approaches or abandoned. Kept for historical reference and to document decision-making process.

*(Currently empty)*

### `/uncommitted/`
Forward-looking design documents and architectural specifications for future iterations. These represent planned systems that haven't been committed to a specific iteration yet.

- **`component-set-01.md`** - MVP gameplay components (Interactable, Container, Door, Switch, etc.) - 15 core components for interactive environments
- **`editor-inspector.md`** - Extensible inspector panel architecture with component-driven sub-panels
- **`enhanced-gizmos.md`** - Advanced gizmo system (multi-select, snapping, constraints)
- **`glb-support.md`** - GLB/GLTF model loading, animation support, material handling
- **`lua-scripting.md`** - Embedded Lua scripting for prefabs and levels (event callbacks, designer empowerment)
- **`model-set-01.md`** - MVP Corporate asset pack (31 models for 2 demo rooms + liminal hallway)
- **`persistence.md`** - Two-database persistence system (static DB for immutable content, dynamic DB for save states)
- **`world_architecture.md`** - Chunk streaming system, LOD, floating origin for large world support

## Document Lifecycle

```
uncommitted/ → iter-XX.md → in-progress/ → implemented/
                                              ↓
                                         superceded/
```

1. **Uncommitted**: Initial design, not yet scheduled
2. **Iteration Plan** (iter-XX.md): Scheduled for specific iteration
3. **In Progress**: Active implementation, may be refined based on learnings
4. **Implemented**: Feature complete, serving as reference documentation
5. **Superceded**: Archived when replaced by newer approaches or abandoned

## Iteration Naming Convention

- **Iteration 0** (iter-00.md): Foundation - Core game systems before editor work begins
- **Iteration 1** (iter-01.md): Editor MVP - Basic scene building tools
- **Iteration 2+**: Future iterations TBD

## Related Documentation

- **`../foundational/`** - Core game design (mechanics, setting, timeline)
- **`../../assets/`** - Asset files (meshes, textures, audio)
- **`../../src/`** - Source code implementation

## Contributing Notes

When creating new specifications:
1. Start in `/uncommitted/` for exploratory design
2. Move to iteration plan when scheduling work
3. Copy to `/in-progress/` when implementation begins
4. Move to `/implemented/` when feature is complete and stable

Keep specifications conceptual and low-level - focus on *what* and *why*, not implementation details.
