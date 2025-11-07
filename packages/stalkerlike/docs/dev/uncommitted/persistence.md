# Persistence Architecture

## Overview
Two-database SQLite architecture separating immutable game content from player state. This enables clean updates, safe saves, and extensive moddability.

## Database Structure

### Static Database (`game_static.db`)
Immutable game content. Replaced entirely on updates.

```sql
-- Module definitions
CREATE TABLE modules (
    id TEXT PRIMARY KEY,
    mesh_path TEXT NOT NULL,
    default_material TEXT,
    collision_type TEXT,
    vertex_colors TEXT  -- JSON array
);

-- Hand-crafted level chunks
CREATE TABLE chunks (
    id TEXT PRIMARY KEY,
    x INTEGER, y INTEGER, z INTEGER,
    template TEXT,
    faction TEXT DEFAULT 'abandoned',
    entities TEXT,  -- JSON blob of placed entities
    UNIQUE(x, y, z)
);
CREATE INDEX idx_chunks_spatial ON chunks(x, y, z);

-- Narrative content
CREATE TABLE datapads (
    id TEXT PRIMARY KEY,
    title TEXT,
    content TEXT,
    location_hint TEXT,
    year_written INTEGER
);

-- Game definitions
CREATE TABLE props (
    id TEXT PRIMARY KEY,
    name TEXT,
    mesh_path TEXT,
    weight REAL,
    value INTEGER,
    components TEXT  -- JSON ECS components
);

-- Scripting
CREATE TABLE scripts (
    id TEXT PRIMARY KEY,
    name TEXT UNIQUE,
    trigger_type TEXT,  -- 'on_enter', 'on_interact', 'on_timer'
    script TEXT         -- Lua code
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
content/
├── schema/
│   ├── static_v1.sql
│   └── dynamic_v1.sql
├── levels/
│   ├── 00_corporate_hub.yaml
│   ├── 01_transit_alpha.yaml
│   └── 99_the_deep.yaml
├── props/
│   └── definitions.yaml
├── narrative/
│   ├── datapads.yaml
│   └── timeline.yaml
└── build.py
```

### YAML to SQL Conversion
```python
# build.py
import yaml
import json
import sqlite3

def yaml_to_sql(yaml_file, table_name):
    with open(yaml_file) as f:
        data = yaml.safe_load(f)
    
    # Convert based on content type
    if table_name == 'chunks':
        return convert_level(data)
    elif table_name == 'props':
        return convert_props(data)
    # ...etc

def convert_level(data):
    chunk = data['chunk']
    entities = json.dumps(data.get('entities', []))
    
    return f"""
        INSERT INTO chunks (id, x, y, z, faction, entities)
        VALUES ('{chunk['id']}', {chunk['position'][0]}, 
                {chunk['position'][1]}, {chunk['position'][2]},
                '{chunk.get('faction', 'abandoned')}', 
                '{entities}');
    """

def build_static_db():
    conn = sqlite3.connect('game_static.db')
    
    # Schema
    conn.executescript(open('schema/static_v1.sql').read())
    
    # Content
    for yaml_file in Path('levels').glob('*.yaml'):
        conn.executescript(yaml_to_sql(yaml_file, 'chunks'))
    
    # Optimize
    conn.execute("VACUUM")
    conn.execute("ANALYZE")
```

## Runtime Usage

### Database Connection
```rust
use rusqlite::{Connection, OpenFlags};

pub struct GameDatabase {
    static_db: Connection,   // Read-only
    dynamic_db: Connection,  // Read-write
}

impl GameDatabase {
    pub fn new(save_slot: u32) -> Result<Self> {
        // Open static DB read-only
        let static_db = Connection::open_with_flags(
            "game_static.db",
            OpenFlags::SQLITE_OPEN_READ_ONLY
        )?;
        
        // Open or create save
        let save_path = format!("saves/slot_{}.db", save_slot);
        let dynamic_db = if Path::new(&save_path).exists() {
            Connection::open(&save_path)?
        } else {
            // Copy template
            std::fs::copy("game_dynamic_template.db", &save_path)?;
            Connection::open(&save_path)?
        };
        
        // Attach for cross-DB queries
        dynamic_db.execute(
            "ATTACH DATABASE 'game_static.db' AS static",
            []
        )?;
        
        Ok(Self { static_db, dynamic_db })
    }
}
```

### Querying
```rust
// Get chunk (check procedural first, then static)
pub fn get_chunk(&self, coords: ChunkCoord) -> Option<ChunkData> {
    // Check generated chunks
    if let Ok(data) = self.dynamic_db.query_row(
        "SELECT entities FROM generated_chunks 
         WHERE x=? AND y=? AND z=? AND save_id=?",
        params![coords.x, coords.y, coords.z, self.save_id],
        |row| row.get::<_, String>(0)
    ) {
        return Some(parse_chunk_data(&data));
    }
    
    // Check static chunks
    if let Ok(data) = self.static_db.query_row(
        "SELECT entities FROM chunks 
         WHERE x=? AND y=? AND z=?",
        params![coords.x, coords.y, coords.z],
        |row| row.get::<_, String>(0)
    ) {
        return Some(parse_chunk_data(&data));
    }
    
    None
}

// Track discovery
pub fn mark_discovered(&self, entity_id: &str, chunk_id: &str) {
    self.dynamic_db.execute(
        "INSERT OR IGNORE INTO discoveries 
         (save_id, entity_id, chunk_id) 
         VALUES (?, ?, ?)",
        params![self.save_id, entity_id, chunk_id]
    ).ok();
}
```

## Update System

### Patching
```bash
#!/bin/bash
# update.sh

# Backup saves
cp -r saves/ "backup_$(date +%s)/"

# Replace static DB (atomic)
mv game_static_new.db game_static.db

# Run migrations if needed
VERSION_OLD=$(sqlite3 saves/slot_1.db "SELECT value FROM settings WHERE key='version'")
VERSION_NEW=$(sqlite3 game_static.db "SELECT version FROM metadata")

if [ -f "migrations/${VERSION_OLD}_to_${VERSION_NEW}.sql" ]; then
    for save in saves/*.db; do
        sqlite3 "$save" < "migrations/${VERSION_OLD}_to_${VERSION_NEW}.sql"
    done
fi
```

### Migration Example
```sql
-- migrations/v1.0_to_v1.1.sql
-- Add new column safely
ALTER TABLE chunk_states ADD COLUMN temperature REAL DEFAULT 20.0;

-- Update version
UPDATE settings SET value = '1.1' WHERE key = 'version';
```

## Git Workflow

### Development
```bash
# Edit human-friendly YAML
vim levels/mining_shaft_7.yaml

# Convert to SQL (pre-commit hook)
python build.py

# Both tracked in git
git add levels/mining_shaft_7.yaml
git add generated/mining_shaft_7.sql

# CI builds game_static.db
make release
```

### Version Control
```
.gitignore:
*.db         # Don't track binary DBs
saves/       # Don't track saves

git-lfs:
*.db filter=lfs  # Or use LFS for binaries
```

## Advantages

1. **Clean Updates**: Replace one file, saves untouched
2. **Moddability**: Players can query/modify databases
3. **Debugging**: SQL queries for game state inspection
4. **Performance**: SQLite handles millions of entities
5. **Compatibility**: Migrations preserve saves across versions
6. **Version Control**: YAML sources are diff-friendly
7. **Hot Reload**: Change YAML, rebuild DB, see changes

## Query Examples

### Analytics
```sql
-- Player progress
SELECT COUNT(DISTINCT chunk_id) as explored_chunks,
       COUNT(*) as total_discoveries,
       MAX(timestamp) as last_played
FROM discoveries WHERE save_id = 1;

-- Death locations
SELECT chunk_id, COUNT(*) as deaths 
FROM events 
WHERE event_type = 'player_death' 
GROUP BY chunk_id 
ORDER BY deaths DESC;
```

### Modding
```sql
-- Add custom content
INSERT INTO static.chunks VALUES ('mod_nightmare', -9999, -9999, -9999, ...);

-- Modify game rules
UPDATE static.props SET value = value * 2;  -- Double all item values

-- Unlock everything
INSERT INTO discoveries SELECT 1, id, 'cheated' FROM static.datapads;
```

## Performance Considerations

- Memory-map static DB for read performance
- Keep frequently accessed data in Bevy ECS
- Use prepared statements for repeated queries
- Index spatial coordinates for range queries
- VACUUM periodically to maintain performance
- Use transactions for bulk operations
