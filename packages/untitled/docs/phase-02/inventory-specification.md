# Inventory System Specification
**Version**: 1.0  
**Target Phase**: Phase 2 - Depth & Extraction

## Overview
A flexible grid-based inventory system that supports the game's extraction mechanics. The system emphasizes limited carrying capacity to create meaningful item choices while providing robust item management, drag-and-drop interface, and scalable item definitions.

## Core Design Principles

### 1. Reusable Architecture
- **Universal Component**: Single inventory system works for players, chests, containers
- **Configurable Constraints**: Different entities can have different capacity limits
- **Flexible Grid Sizes**: Supports various inventory dimensions and layouts
- **Entity-Agnostic Operations**: Core functionality works regardless of owner type

### 2. Limited Capacity Design
- **Meaningful Constraints**: Limited slots create item choice pressure
- **Dynamic Sizing**: Inventory capacity can change based on player progression, equipment, or conditions
- **Grid-Based Placement**: Items occupy physical space with rotation support
- **Strategic Organization**: Spatial arrangement becomes part of inventory management

### 3. Extensible Framework
- **Data-Driven Items**: Item definitions loaded from configuration files
- **Flexible Properties**: Generic system accommodates various item types
- **Modular Systems**: Components can be extended without breaking existing functionality
- **Performance Focused**: Efficient operations for real-time gameplay

## Item System Architecture

### Item Definition Registry
- **Global Item Database**: Central registry loaded at startup containing all item definitions
- **Unique Item Identifiers**: Each item type has a unique ID for efficient lookups
- **Item Groups**: Logical groupings of items (weapons, consumables, etc.) for batch operations
- **Template System**: Item definitions serve as templates for creating item instances

### Item Types & Categories
- **Flexible Categorization**: Support for arbitrary item categories defined at runtime
- **Hierarchical Types**: Items can belong to nested category structures
- **Extensible System**: New item types can be added without code changes
- **Custom Properties**: Each category can define its own relevant attribute sets

### Rarity & Quality System
- **Configurable Tiers**: Support for multiple rarity levels with visual distinction
- **Contextual Spawning**: Items can have different rarity in different areas
- **Value Scaling**: Rarity affects item stats, drop rates, and visual presentation
- **Extensible Framework**: New rarity tiers can be added via data configuration

### Item Properties & Stats
- **Generic Property System**: Flexible key-value pairs for any item attribute
- **Typed Properties**: Support for numeric, boolean, string, and complex data types
- **Conditional Properties**: Properties can be context-dependent or conditional
- **Extensible Framework**: New property types can be added as game evolves

## Grid Inventory System

### Core Inventory Component
- **Universal Inventory**: Single component type that can be attached to any entity
- **Dynamic Grid Configuration**: Width, height can be modified at runtime based on conditions
- **Access Control**: Configurable restrictions (owner-only, team members, public access, key-required)
- **Behavioral Flags**: Toggle rotation, stacking, auto-sorting per inventory
- **Capacity Modifiers**: Support for temporary or permanent inventory size changes

### Grid Implementation
- **2D Cell Matrix**: Two-dimensional array storing item placement information
- **Spatial Positioning**: Each item knows its exact grid coordinates and dimensions
- **Origin Tracking**: Multi-cell items have a designated origin point (top-left corner)
- **Instance Management**: Hash map linking unique item instances to grid positions

### Item Instance System
- **Unique Identifiers**: Each item instance has a globally unique ID
- **Positional Data**: Grid coordinates, rotation state, and dimensions
- **Stack Management**: Support for stackable items with quantity tracking
- **Modification Support**: Items can have durability, upgrades, and custom properties
- **Rotation States**: 0°, 90°, 180°, 270° rotation options for better space utilization

### Inventory Operations
- **Placement Operations**: Try to place item at specific position with collision detection
- **Auto-Placement**: Find optimal position automatically, considering rotation if allowed
- **Item Removal**: Extract items from grid, handling multi-cell cleanup
- **Area Queries**: Check if grid regions are available for placement
- **Organization Tools**: Compact inventory to eliminate gaps, sort by various criteria
- **Error Handling**: Comprehensive error types for failed operations (no space, invalid position, access denied)
- **Sort Options**: By type, rarity, value, or size for better organization

## Standard Inventory Configurations

### Player Inventory
- **Dynamic Capacity**: Base grid size that can expand/contract based on progression and equipment
- **Growth Sources**: Inventory can increase through leveling, finding bags, or equipment bonuses
- **Capacity Penalties**: Debuffs, curses, or removing equipment can temporarily reduce inventory size
- **Manual Organization**: Player controls item placement and rotation
- **Persistent State**: Items remain in inventory across game sessions, capacity changes saved
- **Purpose**: Primary item storage with strategic space management that evolves with character

### Container Inventories (Chests)
- **Variable Sizes**: Different containers have different capacities
- **Shared Access**: Multiple players can interact (for future multiplayer)
- **Auto-Sort Options**: Optional convenience features for organization
- **Purpose**: World storage and item exchange points

### Merchant/Vendor Inventories
- **Large Capacity**: Can hold many items for trading
- **Filtered Access**: Players may only see relevant items
- **Dynamic Stock**: Inventory contents can change over time
- **Purpose**: Item trading and economy interaction

## Item Generation & Loot System

### Loot Tables
- **Definition Rolling**: Loot tables roll for which item definition to instantiate
- **Weighted Probability**: Items have different drop chances based on weights
- **Conditional Drops**: Items can have requirements (floor level, player progression)
- **Quantity Control**: Min/max drop amounts with guaranteed minimum drops
- **Contextual Loot**: Different loot tables for different contexts (enemies, chests, floors)

### Item Factory System
- **Definition Instantiation**: Creates item instances from rolled item definitions
- **Randomized Qualifiers**: Applies random variations within allowable ranges defined by item template
- **Property Randomization**: Stats, durability, and other properties can vary within defined bounds
- **Range-Based Generation**: Each item definition specifies min/max ranges for randomizable attributes
- **Consistent Rolling**: Same seed produces same item variations for reproducible results
- **ID Management**: Ensure all item instances have unique identifiers

## User Interface Design

### Inventory UI Layout
- **Grid Visualization**: Clear cell boundaries with responsive hover states
- **Entity Binding**: Each UI panel is bound to a specific entity's inventory
- **Size Adaptation**: UI scales appropriately for different inventory sizes
- **State Management**: Track open/closed state and active interactions

### Drag and Drop System
- **Visual Feedback**: Ghost images, valid/invalid drop zones, collision previews
- **Multi-Entity Support**: Drag between different inventories (player to chest)
- **Rotation Integration**: Right-click or hotkey rotation during drag operations
- **Cursor Tracking**: Smooth item following with appropriate offset handling

### Event System
- **Interaction Events**: Item pickup, drop, use, rotation events
- **State Events**: Inventory open/close, access attempts
- **System Integration**: Events trigger appropriate game logic responses
- **Error Feedback**: Clear communication of failed operations to player

### Visual Features
- **Grid Visualization**: Clear cell boundaries with hover highlights
- **Item Icons**: High-quality item sprites with rarity-colored borders
- **Tooltips**: Rich tooltips showing item stats, descriptions, and properties
- **Drag Preview**: Ghost image of item being dragged with valid/invalid drop indicators
- **Auto-Arrange**: Visual feedback for automatic inventory organization
- **Search/Filter**: Text-based search and category filtering

## Implementation Strategy

### ✅ Sub-Phase 2.1: Foundation
**Goal**: Basic inventory functionality without UI
1. **Item Registry System**: Define item templates with property ranges
2. **Core Components**: Inventory component with dynamic grid sizing
3. **Item Factory**: Create instances from definitions with randomized qualifiers
4. **Basic Operations**: Add/remove items, check capacity, handle grid placement
5. **Save/Load**: Persist inventory state between sessions

**Deliverable**: Working inventory system accessible via debug commands

### ✅ Sub-Phase 2.2: Basic UI
**Goal**: Minimal functional inventory interface
1. **Grid Visualization**: Render inventory slots and occupied spaces
2. **Item Display**: Show item icons and basic information
3. **Mouse Interaction**: Click to select/move items
4. **Simple Tooltips**: Display item name and basic stats
5. **Open/Close**: Toggle inventory panel with keyboard shortcut

**Deliverable**: Playable inventory management interface

### ✅ Sub-Phase 2.3: Core Features
**Goal**: Complete inventory interaction system
1. **Drag & Drop**: Full drag and drop between slots and inventories
2. **Item Rotation**: Right-click or hotkey to rotate items for better fit
3. **Auto-Placement**: Smart positioning when picking up items
4. **Stacking**: Combine stackable items automatically
5. **Container Support**: Interact with chest and container inventories

**Deliverable**: Polished inventory interactions

### Sub-Phase 2.4: Integration & Polish
**Goal**: Full integration with game systems
1. **Loot Generation**: Connect to floor progression and enemy drops
2. **Dynamic Sizing**: Implement capacity changes from equipment/progression
3. **Item Usage**: Consume/equip items with proper effects
4. **Visual Polish**: Animations, better feedback, rarity colors
5. **Performance**: Optimize for smooth real-time operation

**Deliverable**: Complete inventory system ready for Phase 2 testing

## File Structure
```
src/inventory/
├── mod.rs                 // Public API exports
├── components.rs          // Inventory, ItemInstance components
├── registry.rs            // ItemRegistry, ItemDefinition
├── grid.rs               // InventoryGrid implementation
├── events.rs             // InventoryEvent definitions
├── systems/
│   ├── mod.rs
│   ├── management.rs     // Core inventory operations
│   ├── ui.rs            // UI rendering and interaction
│   ├── drag_drop.rs     // Drag and drop logic
│   └── loot.rs          // Loot generation and drops
├── ui/
│   ├── mod.rs
│   ├── inventory_panel.rs // Main inventory UI
│   ├── item_tooltip.rs   // Item information display
│   └── drag_preview.rs   // Drag operation visualization
└── data/
    ├── items.ron         // Item definitions data file
    └── loot_tables.ron   // Loot table configurations
```

## Phase 2 Integration

### Floor Progression Support
- **Loot Scaling**: Item quality and types improve with floor depth
- **Capacity Constraints**: Limited inventory forces strategic item selection
- **Persistent Storage**: Successfully extracted items saved between runs
- **Risk/Reward Balance**: Better items deeper, but harder to extract with

### Extraction Mechanics
- **Limited Carrying Capacity**: Forces "what do I keep?" decisions during exploration
- **Item Value Learning**: Players discover item worth through scarcity and choice
- **Strategic Planning**: Inventory management becomes part of dive preparation
- **Loss Consequences**: Death means losing current inventory, teaching item value

### Technical Requirements
- **Save/Load Support**: Inventory state persists between game sessions
- **Network Ready**: Architecture supports future multiplayer expansion
- **Performance Optimized**: Efficient for real-time inventory operations
- **Modding Friendly**: Data-driven design allows community item creation

This specification provides a solid, pragmatic foundation for Phase 2's extraction mechanics while remaining focused on core inventory functionality.
