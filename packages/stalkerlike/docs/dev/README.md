# Development Documentation

This directory contains technical specifications, iteration plans, and design documents for Stalkerlike editor and game development.

## Current Development

**Iteration 1 (In Progress)**: Editor MVP - Level authoring tools
- See `iter-01.md` for detailed task breakdown and completion status
- Week 4 polish phase - autosave, bug fixes, keyboard shortcuts

**Status**: 12/9 core systems complete (exceeded scope!)
- Transform gizmos (translate, rotate, scale)
- Multi-select and grouping
- Hierarchy panel with inline editing
- Scene persistence (YAML)
- Autosave with recovery
- Duplicate/delete operations

## Iteration Plans

- **`iter-00.md`** - Iteration 0: Game mode foundations (player controller, basic 3D scene, save/load)
- **`iter-01.md`** - Iteration 1: Editor MVP (camera, gizmos, primitives, inspector) - **ACTIVE**

## Directory Structure

### `/implemented/`
Completed features serving as reference documentation.
- `outline.md` - Selection outline system using inverted normals

### `/in-progress/`
Active development specifications being implemented or refined.
- `art_pipeline.md` - Low-poly aesthetic, vertex colors, poly budget
- `editor.md` - Full editor vision and feature roadmap

### `/uncommitted/`
Forward-looking designs for future iterations (not yet scheduled).
- `component-set-01.md` - MVP gameplay components (Interactable, Container, Door, Switch)
- `editor-inspector.md` - Extensible inspector with component-driven sub-panels
- `enhanced-gizmos.md` - Advanced gizmo features (multi-select, snapping, constraints)
- `glb-support.md` - GLB/GLTF loading, animation, materials
- `lua-scripting.md` - Embedded Lua for prefabs/levels (event callbacks)
- `model-set-01.md` - MVP Corporate asset pack (31 models for demo rooms)
- `persistence.md` - Two-database system (static/dynamic split)
- `world_architecture.md` - Chunk streaming, LOD, floating origin

### `/superceded/`
Archived specifications replaced by newer approaches. Kept for historical reference.

## Document Lifecycle

```
uncommitted/ → iter-XX.md → in-progress/ → implemented/
                                              ↓
                                         superceded/
```

1. **Uncommitted**: Initial design, not yet scheduled
2. **Iteration Plan**: Scheduled for specific iteration (iter-XX.md)
3. **In Progress**: Active implementation with refinements
4. **Implemented**: Feature complete, reference documentation
5. **Superceded**: Archived when replaced or abandoned

## Related Documentation

- **`../foundational/`** - Game world, setting, mechanics, factions
- **`../../README.md`** - Stalkerlike project overview
- **Root `/AGENTS.md`** - High-level project orientation

## Working with Specifications

When creating new specifications:
1. Start in `/uncommitted/` for exploratory design
2. Add to iteration plan (iter-XX.md) when scheduling
3. Copy to `/in-progress/` during implementation
4. Move to `/implemented/` when complete and stable
5. Archive to `/superceded/` if replaced

Keep specifications conceptual - focus on *what* and *why*, not implementation details.
