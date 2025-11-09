# Persistence Architecture

## Overview
Two-database SQLite architecture separating immutable game content from player state. This enables clean updates, safe saves, and extensive moddability.

## Database Structure

### Static Database (`game_static.db`)
Immutable game content. Replaced entirely on updates.

Illustrative, not final:

```sql
-- Visual atoms (models/)
CREATE TABLE models (
    id TEXT PRIMARY KEY,
    path TEXT NOT NULL,  -- e.g., "models/chest.glb"
    mesh_path TEXT NOT NULL,
    default_material TEXT,
    collision_type TEXT,
    vertex_colors TEXT  -- JSON array
);

-- Functional atoms (component definitions)
CREATE TABLE components (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,  -- e.g., "Interactable", "Container"
    rust_type TEXT NOT NULL,
    default_config TEXT  -- JSON schema
);

-- Prefabs (molecules from prefabs/)
CREATE TABLE prefabs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,  -- e.g., "prefabs/chest_common"
    model_id TEXT,  -- References models.id
    components TEXT,  -- JSON array of component configs
    children TEXT,  -- JSON array of child entities
    scripts TEXT,  -- JSON map of script files
    FOREIGN KEY (model_id) REFERENCES models(id)
);

-- Hand-crafted levels (from levels/)
CREATE TABLE levels (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    path TEXT NOT NULL,  -- e.g., "levels/corporate_hub"
    chunk_x INTEGER, 
    chunk_y INTEGER, 
    chunk_z INTEGER,
    chunk_size_x INTEGER,
    chunk_size_y INTEGER,
    chunk_size_z INTEGER,
    faction TEXT DEFAULT 'abandoned',
    always_loaded BOOLEAN DEFAULT false,
    connections TEXT,  -- JSON array of connection points
    prefab_instances TEXT,  -- JSON array of placed prefabs
    UNIQUE(chunk_x, chunk_y, chunk_z)
);
CREATE INDEX idx_levels_spatial ON levels(chunk_x, chunk_y, chunk_z);

-- Narrative content
CREATE TABLE datapads (
    id TEXT PRIMARY KEY,
    title TEXT,
    content TEXT,
    location_hint TEXT,
    year_written INTEGER
);

-- Metadata
CREATE TABLE metadata (
    version TEXT PRIMARY KEY,
    build_date TEXT,
    content_hash TEXT
);
```

### Dynamic Database (`game_dynamic.db`)
Player state and runtime data. Persists across updates.

```sql
-- Save game info
CREATE TABLE saves (
    id INTEGER PRIMARY KEY,
    name TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    playtime_seconds INTEGER,
    current_chunk TEXT,
    player_state TEXT  -- JSON blob
);

-- Chunk runtime state
CREATE TABLE chunk_states (
    chunk_id TEXT,
    save_id INTEGER,
    last_visited TIMESTAMP,
    power_level REAL DEFAULT 1.0,
    faction_control TEXT,
    custom_state TEXT,  -- JSON for runtime changes
    PRIMARY KEY (chunk_id, save_id)
);

-- Player discoveries
CREATE TABLE discoveries (
    save_id INTEGER,
    entity_id TEXT,
    discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    chunk_id TEXT,
    PRIMARY KEY (save_id, entity_id)
);

-- Procedurally generated content
CREATE TABLE generated_chunks (
    x INTEGER, y INTEGER, z INTEGER,
    save_id INTEGER,
    seed INTEGER,
    generated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    entities TEXT,
    PRIMARY KEY (x, y, z, save_id)
);

-- Event log for narrative
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    save_id INTEGER,
    timestamp REAL,
    chunk_id TEXT,
    event_type TEXT,
    data TEXT  -- JSON
);

-- Settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT
);
```

## Build Pipeline

### Directory Structure
```
assets/
├── models/          # Visual atoms (.glb files)
├── prefabs/         # Molecules (directories with YAML + scripts)
├── levels/          # Organisms (directories with YAML + scripts)
└── build_tool       # Converts YAMLs to database
```

### Content Processing

**Build Process:**
1. Scan `models/` directory and index all .glb files
2. Parse all `prefabs/*/prefab.yaml` files and convert to database entries
3. Parse all `levels/*/level.yaml` files and convert to database entries
4. Process narrative content (datapads, lore)
5. Optimize database (indexing, vacuuming)
6. Generate `game_static.db`

**Prefab Conversion:**
- Read prefab.yaml structure
- Extract model references, components, children
- Store as single database row with JSON blobs for complex data
- Link to model IDs and script paths

**Level Conversion:**
- Read level.yaml with all placed prefab instances
- Calculate chunk coordinates from world position
- Store prefab placements, spawn points, triggers
- Preserve connection points for procedural generation
- Link scripts and narrative elements

## Runtime Usage

### Database Access

**Dual Database System:**
- **Static DB**: Read-only, contains all game content, replaced on updates
- **Dynamic DB**: Read-write, contains player saves and runtime state
- Databases can be queried together using ATTACH

**Query Flow:**
1. Check dynamic DB for generated/modified chunks
2. Fall back to static DB for hand-authored content
3. Cache frequently accessed data in memory
4. Track player discoveries and modifications in dynamic DB

**Key Operations:**
- Load level data by chunk coordinates
- Retrieve prefab definitions by ID
- Query model paths for asset loading
- Track player discoveries across chunks
- Store runtime modifications (doors opened, items taken, etc.)

## Update System

### Patching Strategy

**Update Process:**
1. Backup all player saves before update
2. Replace `game_static.db` atomically (single file swap)
3. Check version compatibility between old saves and new content
4. Run database migrations if schema changed
5. Verify save file integrity

**Migration System:**
- Version-specific SQL scripts for schema changes
- Applied to dynamic DB only (saves)
- Can add columns, update defaults, transform data
- Preserves player progress across updates

**Benefits:**
- Player saves never touched during content updates
- Rollback is just restoring old game_static.db
- Moddable: users can replace database or add content
- Fast updates: replace one file instead of many

## Git Workflow

### Development
Content creators work with human-friendly formats:
- Artists export .glb files to `models/`
- Designers create prefabs in editor → saves to `prefabs/*/prefab.yaml`
- Level designers place prefabs → saves to `levels/*/level.yaml`
- Build tool converts everything to `game_static.db`

### Version Control
```
.gitignore:
game_static.db     # Don't track built DB (regenerate from YAMLs)
game_dynamic*.db   # Don't track saves
saves/             # Don't track saves

Tracked:
models/**/*.glb    # All visual atoms
prefabs/**/        # All prefab directories
levels/**/         # All level directories
build_tool         # Database builder
schema/            # DB schemas
```

**Workflow Benefits:**
- Artists/designers work in familiar formats
- Git tracks source files, not generated database
- Merge conflicts rare (YAMLs in separate directories)
- CI/CD builds database automatically
- Easy to see what changed in code review

## Advantages

1. **Clean Updates**: Replace one file (game_static.db), saves untouched
2. **Moddability**: Players can add models/, prefabs/, levels/ directories
3. **Debugging**: SQL queries for game state inspection
4. **Performance**: SQLite handles millions of entities efficiently
5. **Compatibility**: Migrations preserve saves across versions
6. **Version Control**: YAML/GLB sources are diff-friendly and mergeable
7. **Hot Reload**: Change YAML, rebuild DB, see changes immediately
8. **Shared Vocabulary**: Prefabs used by editor, procedural gen, and runtime
9. **Artist-Friendly**: Artists work in Blender, export .glb, done
10. **Designer-Friendly**: Designers use editor GUI, saves to readable YAML

## Query Examples

### Analytics
Example queries for tracking player behavior:
- Count explored chunks and discovered items per save
- Find most dangerous areas (death heatmap by chunk)
- Track time spent per depth zone
- Identify rarely-discovered secrets

### Modding
Players/modders can extend the game by:
- Adding custom .glb models to `mods/models/`
- Creating new prefabs referencing those models
- Designing complete custom levels
- Inserting new datapads, lore, narratives
- Modifying loot tables and spawn rates
- All via direct SQL or tool-assisted workflows

**Modding Approach:**
- Drop content into appropriate directories
- Run build tool to regenerate database
- Or directly insert into database tables
- Game loads mods alongside base content

## Performance Considerations

- Memory-map static DB for fast read performance
- Keep frequently accessed data in game ECS (not queried every frame)
- Use prepared/cached queries for repeated lookups
- Index spatial coordinates for efficient range queries
- Vacuum dynamic DB periodically to reclaim space
- Use transactions for bulk operations (batch inserts/updates)
- Lazy-load level data only when chunk becomes active

**Memory Strategy:**
- Static DB stays memory-mapped (read-only, safe)
- Prefab definitions cached after first load
- Model paths cached in asset manager
- Level data loaded on-demand per chunk
- Player state kept in ECS, persisted to DB on save events
