Welcome to Learning Rust!

This is a monorepo for hobby game development projects built with Rust. I'm working on 
multiple independent games to learn Rust and game development using 
[Bevy](https://raw.githubusercontent.com/bevyengine/bevy/refs/heads/main/README.md) and
[Rapier](https://raw.githubusercontent.com/dimforge/bevy_rapier/refs/heads/master/README.md) 
as ECS and physics engines.

## Current Project State

**Active Development**: `packages/stalkerlike/` - 3D level editor and survival horror game
- Underground colony survival horror game set on Ross 154a
- Level editor built with Bevy 0.16 (Iteration 1 in progress)
- See `packages/stalkerlike/README.md` for editor context and current priorities
- See `packages/stalkerlike/docs/foundational/README.md` for game world and design

**Other Projects**:
- `packages/untitled/` - Separate game prototype (independent, unrelated to Stalkerlike)
- `packages/sanity/` - Rust learning experiments
- `packages/shadowcast/` - Line-of-sight algorithm tests
- `packages/generation/` - Procedural generation experiments

## Working with This Project

Review recent git commits for context:
```bash
git log --oneline -10
```

**Key Documentation**:
- `packages/stalkerlike/README.md` - Stalkerlike project overview
- `packages/stalkerlike/docs/foundational/README.md` - Game world, setting, mechanics
- `packages/stalkerlike/docs/dev/iter-01.md` - Current editor development iteration
- `packages/untitled/README.md` - Untitled game overview (separate project)

**Technical Context**:
- Bevy 0.16 APIs (observers, picking system, component hooks)
- Stalkerlike editor systems in `packages/stalkerlike/src/editor/`
- Stalkerlike game systems in `packages/stalkerlike/src/game/`
- Run Stalkerlike game: `cargo run --package stalkerlike`
- Run Stalkerlike editor: `cargo run --package stalkerlike -- --editor`
- Run Untitled game: `cargo run --package untitled`

