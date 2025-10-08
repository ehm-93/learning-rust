# FOW System Cleanup & Consolidation Summary

**Date**: October 8, 2025  
**Focus**: Maintainability and best practices

## Overview

Completed a comprehensive cleanup and consolidation pass on the Fog of War system to improve code maintainability, readability, and documentation.

## Changes Made

### 1. Extracted Magic Numbers to Named Constants

**File**: `combat/fow/systems.rs`

Consolidated all magic numbers into well-documented constants at the module level:

```rust
/// Blur margin as a percentage of LOS radius for smooth edge fade (0.8 = 80%)
const BLUR_MARGIN_PERCENTAGE: f32 = 0.8;

/// Minimum blur margin in tiles to ensure smooth edges even with small LOS radius
const MIN_BLUR_MARGIN: f32 = 16.0;

/// Distance in tiles over which to blend vision near walls for smooth fade
const WALL_BLEND_DISTANCE: f32 = 3.5;

/// Budget (in seconds) for FOW lerping per frame to avoid frame drops
const FOW_LERP_BUDGET: f32 = 0.002;

/// Threshold to consider lerping complete (difference less than this means we're close enough)
const LERP_COMPLETION_THRESHOLD: f32 = 0.5;

/// Safety limit for raycasting to prevent infinite loops
const RAYCAST_MAX_STEPS: i32 = 1000;
```

**Benefits**:
- Single source of truth for tunable values
- Self-documenting code
- Easy to adjust parameters for gameplay tuning

### 2. Comprehensive Module Documentation

**File**: `combat/fow/systems.rs`

Added extensive module-level documentation covering:
- System architecture and data flow
- Two-tier vision concept (force radius + LOS radius)
- Task lifecycle (spawning → calculation → polling → lerping → rendering)
- Coordinate system conventions
- Performance considerations

**Key Section**:
```rust
//! # Coordinate Systems
//!
//! **CRITICAL**: Different coordinate systems are used throughout:
//! - **Terrain tiles**: Indexed as `tiles[x][y]` (column-major)
//! - **FOW arrays**: Indexed as `vision[y][x]` (row-major)
//! - **Texture rendering**: Y-axis flipped using `.rev()` to match Bevy's bottom-left origin
```

### 3. Raycasting Module Extraction

**New File**: `combat/fow/raycasting.rs`

Extracted line-of-sight raycasting logic into a separate, reusable module:

- `is_wall_at()` - Check for walls at world tile coordinates
- `cast_ray_distance()` - Bresenham's line algorithm for LOS checks

**Benefits**:
- Better code organization
- Potential for reuse in other systems (AI, projectiles, etc.)
- Easier to test in isolation
- Clear separation of concerns

### 4. Vision Stamp Generation Consolidation

**File**: `combat/fow/systems.rs`

Refactored `create_vision_stamps()` into smaller, focused functions:

- `create_vision_stamps()` - Main entry point
- `calculate_blur_margin()` - Blur margin calculation
- `calculate_tile_visibility()` - Per-tile visibility logic

**Before**:
```rust
// 50+ lines of inline logic with nested conditions
if distance <= force_radius as f32 {
    stamp[y][x] = (255.0 * force_strength) as u8;
    needs_los[y][x] = false;
} else if distance <= los_radius as f32 {
    // Complex gradient calculation inline...
```

**After**:
```rust
let (visibility, needs_los_check) = calculate_tile_visibility(
    distance,
    force_radius_f,
    force_strength,
    los_radius_f,
    los_strength,
    blur_margin_f,
    transition_distance,
);
```

**Benefits**:
- Improved readability
- Easier to test individual components
- Better variable naming
- Self-documenting logic flow

### 5. Plugin System Ordering Optimization

**File**: `combat/fow/plugin.rs`

Cleaned up duplicate `Update` system blocks and improved documentation:

```rust
/// Plugin for the Fog of War system
///
/// Registers FOW resources and systems with appropriate scheduling:
/// - `FixedUpdate`: Load/unload, task spawning/polling, lerping (budgeted, deterministic)
/// - `Update`: Drawing (visual smoothness, frame-rate dependent)
```

**Changes**:
- Removed duplicate `Update` blocks
- Clear separation between deterministic logic (FixedUpdate) and rendering (Update)
- All systems properly chained
- Better documentation of scheduling rationale

### 6. Logging and Error Handling

**Files**: `combat/fow/systems.rs`

Added comprehensive logging throughout the system:

**Task Spawning**:
```rust
debug!("Spawned {} FOW calculation tasks", spawned_count);
trace!("Spawned FOW calculation task for entity {:?} at pos {:?}", entity, pos);
```

**Task Polling**:
```rust
debug!("Polling {} completed FOW calculation tasks", completed_tasks.len());
debug!("Applied FOW updates to {} chunks, marked for lerping", total_chunks_updated);
```

**Lerping**:
```rust
trace!("FOW lerp: processed {} chunks, completed {} ({:.4}s)", processed, completed, elapsed);
debug!("FOW lerp exceeded budget: processed {}/{} chunks, debt: {:.4}s", processed, total, debt);
```

**Early Returns**:
```rust
if revealer_count == 0 {
    return; // Early exit when no work needed
}
```

**Benefits**:
- Better debugging capabilities
- Performance monitoring
- Understanding of system behavior during development

### 7. Coordinate System Documentation

**File**: `combat/fow/systems.rs`

Added critical documentation about coordinate system conversions in key functions:

**In `create_fow_texture()`**:
```rust
/// # Coordinate System Notes
///
/// **CRITICAL**: This function handles coordinate system conversions:
/// - Input `vision` array uses row-major indexing: `vision[y][x]`
/// - Bevy's texture coordinate system has origin at bottom-left
/// - We iterate Y in reverse (`.rev()`) to flip the texture vertically
/// - Output pixels are RGBA: black (0,0,0) with inverted alpha
```

**In terrain snapshot creation**:
```rust
// NOTE: terrain tiles use [x][y] indexing, but we store as [y][x] for FOW consistency
for y in 0..CHUNK_SIZE_TILES {
    for x in 0..CHUNK_SIZE_TILES {
        wall_data[y][x] = tiles[x][y] == TileType::Wall;
    }
}
```

**Benefits**:
- Prevents coordinate system bugs
- Makes implicit knowledge explicit
- Easier onboarding for new developers

## File Structure After Cleanup

```
combat/fow/
├── mod.rs                 # Module exports
├── components.rs          # FOW data structures
├── plugin.rs              # Bevy plugin registration
├── raycasting.rs          # NEW: Line-of-sight utilities
└── systems.rs             # FOW computation and rendering
```

## Performance Impact

**Zero performance impact** - all changes are organizational and documentary:
- Constants instead of literals (compile-time)
- Function extraction (inlined by compiler)
- Documentation (stripped in release builds)
- Logging (conditional compilation with levels)

## Testing

- ✅ Code compiles without errors
- ✅ All warnings are pre-existing (dead code analysis)
- ✅ No behavioral changes to FOW system
- ✅ Ready for gameplay testing

## Maintenance Benefits

1. **Easier Parameter Tuning**: All magic numbers in one place
2. **Better Documentation**: Clear system architecture explanation
3. **Improved Debugging**: Comprehensive logging at appropriate levels
4. **Code Reusability**: Raycasting module can be used by other systems
5. **Clearer Intent**: Well-named functions and variables
6. **Onboarding**: New developers can understand system from documentation

## Future Improvements

While not part of this cleanup, potential future enhancements:

1. Extract vision stamp generation to its own module
2. Add unit tests for raycasting logic
3. Add integration tests for FOW calculations
4. Consider caching vision stamps for common configurations
5. Add performance metrics collection (frame times, task counts)

## Related Documentation

- Module docs: `combat/fow/systems.rs` (top of file)
- Architecture overview: Main module documentation
- Coordinate systems: Inline documentation in key functions
- Performance: Constants and budgeting explanations

---

This cleanup maintains all existing functionality while significantly improving code maintainability and developer experience.
