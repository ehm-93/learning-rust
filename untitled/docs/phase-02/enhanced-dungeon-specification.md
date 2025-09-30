# Enhanced Dungeon System Specification

## Overview

This enhanced dungeon system serves the core extraction paradigm by transforming every mechanic into a question: **"How deep will you go, and what will you sacrifice to return?"** 

Success is measured not by reaching depths, but by returning to the Cathedral with valuable discoveries. Every system reinforces meaningful choices under pressure while maintaining the philosophy of discovery over instruction.

## Core Requirements

### 1. The Cathedral
**The only truly safe space and the nexus of choice**

The Cathedral represents **successful extraction** and serves as the **risk negotiation center**. Three portals display the available starting depths with their current modifiers - stable combinations that persist across visits. This creates the fundamental choice: which risk/reward profile matches your current goals?

**The Portal Choice Philosophy**:
- Three doors show different modifier combinations for your chosen starting depth
- Modifiers remain consistent - level 10 with modifiers A, B, C stays that way until you change it
- **Currency-based modification**: Spend extracted resources to reroll, add, or clear modifiers
- Choice becomes an **investment decision** - do you spend currency to optimize risk, or accept the given challenge?

**The Cathedral's Role in Discovery**:
- Extracted loot reveals its true purpose and can become modification currency
- Portal modifiers hint at deeper mechanics without explicit explanation
- The modification system itself must be discovered through experimentation

### 2. The Geography of Commitment
**Each step deeper changes the world behind you**

Levels persist during descent, creating a geography where safety exists behind you, not ahead. **Greater sanctuaries** (every 10th level) become unreachable once abandoned - comfort that vanishes the moment you leave it. **Lesser sanctuaries** appear after every level, offering limited evacuation opportunities.

Persistent changes accumulate evidence of your journey: battle damage, cleared rooms, moved objects. The world remembers your passage, making extraction feel like escaping from a fundamentally altered environment.

### 3. Extraction Economics
**Value exists only when successfully returned to the surface**

Loot has no inherent value - only extraction potential. Limited carrying capacity creates **commitment friction**: "Is this new discovery worth abandoning something I already have?" Deeper discoveries offer exponentially greater rewards at exponentially greater extraction risk, creating natural greed pressure where players consistently push beyond their comfort zone.

**Sawtooth Scaling**: Knowledge shortcuts create **reward and difficulty penalties** to prevent exploitation. Level 20 offers superior loot and appropriate challenge, while level 21 provides reduced rewards at lower difficulty. Both scale linearly until level 25 equals level 20. This forces players to "earn back" their progression through sequential descent, making shortcuts feel like **convenience, not advantage**.

**Modifier Extremification**: After completing a full 10-level flight (e.g., levels 1-10), revisiting those levels reveals **intensified modifiers**. Previously mild effects become more pronounced, creating a **new game plus** experience where familiar territories offer fresh challenges for experienced players.

**Progressive Clearing Rewards**: 
- **Room clearing** provides immediate loot drops scaled to current depth
- **Level completion** grants bonus currency and guaranteed rare items
- **Perfect clearing** (all rooms) multiplies extraction rewards by depth-appropriate factors
- **Abandoned rooms** forfeit their rewards permanently, creating pressure to explore thoroughly

### 4. Sanctuary Crossroads
**Choice chambers that shape your deeper commitment**

**Lesser Sanctuaries** (after every level): Three onward paths with visible modifiers, one **limited evacuation portal** (restricted inventory), and a single reward chest. You can spend currency to modify the paths or reroll the chest before opening.

**Greater Sanctuaries** (levels 10, 20, 30+): The same path choices plus **full evacuation portal** and **three reward chests** that can each be individually modified. These represent major investment opportunities - spend heavily to guarantee powerful rewards, or accept the given odds.

**The Investment Dilemma**: Every sanctuary asks whether to spend precious currency optimizing your path forward, or save it for better opportunities deeper down. Lesser sanctuaries offer **emergency escape** with inventory penalties, while greater sanctuaries provide **safe extraction** with full rewards.

### 5. The Currency of Risk
**Extracted resources become tools of negotiation**

Extracted items serve dual purposes: equipment upgrades and **modification currency**. Rare orbs, essences, and artifacts can reroll path modifiers, add beneficial effects, or clear unwanted complications. This creates **economic pressure** - every powerful modifier tool used is one fewer available for future optimization.

**The Modification Philosophy**:
- **Reroll Currency**: Change the entire modifier set for a path or chest
- **Additive Currency**: Layer additional modifiers onto existing ones
- **Cleansing Currency**: Remove specific unwanted modifiers
- **Augmentation Currency**: Guarantee certain types of modifiers appear

Never explained explicitly - players discover through experimentation what each currency does, sharing knowledge through community interaction.

### 6. Sanctuary Rewards
**Guaranteed treasures that can be shaped by investment**

**Lesser Sanctuary Chests** (every level): Single reward container with visible potential outcomes. Spend currency to reroll contents, add guaranteed item types, or increase rarity tiers. The chest becomes an **investment target** - how much are you willing to spend for better odds?

**Greater Sanctuary Vaults** (every 10 levels): Three independent chests, each modifiable separately. This creates **portfolio management** - spread currency across all three for consistent rewards, or focus investment on one chest for maximum potential.

**The Chest Gamble**: Unlike random loot drops, sanctuary chests can be seen and modified before opening. This transforms luck into **calculated investment**, where players can spend currency to tilt odds in their favor.

### 7. Knowledge as Progression
**Successful extraction grants wisdom**

Only those who successfully extract from deep levels gain knowledge shortcuts to access those depths directly. This makes progression feel genuinely earned rather than arbitrary, reinforcing extraction as the true measure of success while supporting emergent community strategies.

**The Depth Limit**: Level 100 represents the ultimate challenge threshold - the highest starting point accessible through knowledge shortcuts. Beyond this, only sequential descent is possible, preserving the most extreme depths for those willing to undertake the full journey.

### 8. Extensible Generation Architecture
**Algorithmic variety that scales with depth**

**Generation Rotation**: Different algorithms activate every 10 levels, creating distinct thematic and mechanical experiences. Each algorithm defines room layouts, enemy spawning patterns, environmental hazards, and architectural styles.

**Modular Implementation**: New generation algorithms can be added without disrupting existing content. Each algorithm exposes configuration parameters that interact with the modifier system, allowing player currency to influence generation within algorithmic constraints.

**Environmental Storytelling**: Generation algorithms tell stories through architecture - ancient ruins suggest lost civilizations, crystalline caverns hint at magical corruption, organic passages reveal living dungeons. Each depth range becomes a chapter in the world's archaeological narrative.

---

## The Modifier Flow

### Surface Preparation
1. **Choose starting depth** from unlocked levels
2. **View three portal options** with stable modifier combinations
3. **Invest currency** to reroll, add, or clear modifiers across all three options
4. **Select your preferred risk/reward profile** and enter

### Sanctuary Decisions
1. **Arrive at sanctuary** after clearing each level
2. **View three onward paths** with their modifier combinations  
3. **Examine reward chest(s)**: 1 chest (lesser) or 3 chests (greater)
4. **Invest currency** to modify paths, chests, or both
5. **Choose**: Modified path forward, limited evac (lesser), or full evac (greater)

### Investment Psychology
**Frequent Decisions**: Lesser sanctuaries after every level create constant economic pressure
**Major Milestones**: Greater sanctuaries every 10 levels offer significant investment opportunities
**Emergency Calculation**: Limited evacuation available constantly, but at inventory cost
**Strategic Timing**: Save currency for greater sanctuaries, or spend incrementally at lesser ones?

This system transforms "difficulty scaling" into **active economic planning** where players make investment decisions after every single level, with major portfolio decisions every 10 levels. **Generation algorithms** respond to applied modifiers, creating emergent combinations of environmental design and player-chosen challenge parameters.

---

## Design Principles

**Extraction-First**: Every system serves the core "extract or perish" tension, creating meaningful choices about risk, reward, and when to retreat to safety.

**Discovery Over Instruction**: Players learn through experience - discovering sanctuary rules through gameplay, modifier effects through experimentation, sharing knowledge organically within the community.

**Environmental Storytelling**: The world tells its story through persistence and change. Battle damage, cleared rooms, and unreachable sanctuaries create narrative from player action rather than scripted events.

---

## Technical Requirements

### Core Data Structures

**Cathedral State**
- Portal configurations with stable modifier sets per depth
- Player progression tracking (max depth reached, unlocked shortcuts)
- Currency inventory and modification history
- NPC and artifact display state

**Level Persistence System**
- Level state cache during active runs (room clearing, enemy positions, item drops)
- Generation seed storage per level for consistent regeneration
- Modifier application state per level
- Reset triggers on Cathedral return or death

**Sanctuary Management**
- Lesser sanctuary: 3 path options, 1 chest, limited evac portal
- Greater sanctuary: 3 path options, 3 chests, full evac portal
- Currency modification interface for paths and chests
- Evacuation inventory filtering system

### Modifier System Architecture

**Currency Types**
- Reroll orbs (complete modifier set changes)
- Additive essences (layer new modifiers)
- Cleansing crystals (remove specific modifiers)
- Augmentation runes (guarantee modifier types)

**Modifier Application**
- Path modifier persistence (stable until player changes)
- Chest modifier previews before opening
- Extremification tracking (completed flight → intensified modifiers)
- Generation algorithm interaction with applied modifiers

**Economic Pressure Systems**
- Currency scarcity scaling with depth
- Investment decision points at every sanctuary
- Emergency vs. strategic spending mechanics

### Progression Systems

**Sawtooth Scaling Implementation**
- Reward reduction formula for jumped-to levels (level 21 < level 20)
- Difficulty reduction formula for jumped-to levels
- Linear recovery progression (levels 21-25 scaling back to level 20 equivalence)
- Tracking system for legitimate vs. shortcut progression

**Knowledge Shortcuts**
- Extraction verification system (successful return to Cathedral)
- Portal unlock mechanism (k*10+1 levels accessible after reaching k*10+5)
- Level 100 cap enforcement for shortcuts
- Beyond-100 sequential-only progression

### Reward Distribution

**Clearing Rewards**
- Room clearing: immediate depth-scaled loot drops
- Level completion: bonus currency and guaranteed rares
- Perfect clearing: extraction multiplier application
- Abandoned room penalty: permanent reward forfeiture

**Sanctuary Chest System**
- Visible outcome previews before modification
- Currency-based outcome manipulation
- Portfolio management for greater sanctuary vaults
- Risk/reward calculation display

### Generation Framework

**Algorithm Rotation**
- 10-level generation cycles with distinct themes
- Modular algorithm registration system
- Configuration parameter exposure for modifier interaction
- Environmental storytelling through architectural choices

**Level State Management**
- Persistent changes during single runs (battle damage, cleared rooms)
- Complete regeneration on Cathedral return
- Modifier extremification after flight completion
- Archaeological narrative progression through depth ranges

**Integration Points**
- Combat system interaction with generated environments
- Inventory system integration with clearing rewards
- Audio/visual theme coordination with generation algorithms
- UI adaptation for different generation styles

### Dependencies & Constraints

**Asset Requirements**
- No visual assets currently exist - all phases will require placeholder meshes initially
- Cathedral architecture, portal designs, sanctuary environments need placeholder geometry
- Currency items, chest models, UI elements require simple placeholder representations
- Generation algorithms can use basic geometric shapes until proper environmental assets available

**Enemy System Limitations**
- Current implementation may require placeholder enemies for initial phases
- Comprehensive enemy system (varied behaviors, depth scaling, modifier responsiveness) needed for full experience
- Modifier-influenced enemy spawning requires expanded enemy archetypes
- Environmental storytelling through enemies needs diverse enemy types per generation algorithm

**Development Strategy**
- All phases can proceed with placeholder meshes and basic geometric assets
- Phase 1-2 can proceed with existing enemy system and placeholders
- Phase 3+ benefits significantly from expanded enemy variety and modifier interaction
- Consider parallel asset creation and enemy system development alongside core systems

### Implementation Phases

**Phase 1: Core Cathedral & Sanctuary System**

*Phase 1a: Cathedral Foundation*
- Basic Cathedral scene with three portal archways
- Portal activation/selection mechanics
- Simple modifier display (text-based initially)
- Player spawn/return system
**Deliverable**: Playable Cathedral hub where players can see and select from 3 portals with basic modifier text display

*Phase 1b: Basic Currency System*
- Simple reroll orbs as primary currency type
- Currency inventory management
- Basic portal modifier rerolling
- Currency drop from enemies/chests
**Deliverable**: Working currency system where players collect orbs and can reroll portal modifiers in Cathedral

*Phase 1c: Lesser Sanctuaries*
- Sanctuary spawn after each level completion
- Three path options with visible modifiers
- Single reward chest with modification capability
- Basic evacuation portal with inventory limits
**Deliverable**: Lesser sanctuary appears after every level with 3 paths, 1 chest, evacuation option, and currency modification

*Phase 1d: Level Persistence*
- Single-run level state caching
- Room clearing state preservation
- Basic reset on Cathedral return
- Persistent battle damage and moved objects
**Deliverable**: Levels remember cleared rooms and damage during single run, reset completely on Cathedral return

**Phase 2: Knowledge Progression & Scaling**

*Phase 2a: Extraction Tracking*
- Successful extraction detection system
- Max depth reached tracking
- Cathedral return verification
- Basic progression save/load
**Deliverable**: System tracks deepest successful extraction and persists progression data across sessions

*Phase 2b: Knowledge Shortcuts*
- Portal unlock system (k*10+1 access after k*10+5 completion)
- Portal availability display in Cathedral
- Level 100 shortcut cap implementation
- Beyond-100 sequential enforcement
**Deliverable**: Cathedral shows unlocked portals (levels 11, 21, 31, etc.) based on extraction achievements, capped at 100

*Phase 2c: Sawtooth Scaling*
- Reward reduction formulas for jumped levels
- Difficulty scaling penalties for shortcuts
- Linear recovery progression (5-level catchup)
- Balance testing and adjustment
**Deliverable**: Jumping to level 21 provides worse rewards/easier difficulty than level 20, scaling linearly back to parity by level 25

*Phase 2d: Greater Sanctuaries*
- Every-10-levels sanctuary detection
- Triple reward chest system
- Full evacuation portal implementation
- Enhanced investment decision interface
**Deliverable**: Levels 10, 20, 30+ spawn greater sanctuaries with 3 chests and full evacuation capability

**Phase 3: Advanced Modifier Systems**

*Phase 3a: Currency Expansion*
- Additive essences (layer modifiers)
- Cleansing crystals (remove specific modifiers)
- Augmentation runes (guarantee types)
- Currency combination and interaction rules
**Deliverable**: Four distinct currency types with different modifier effects, discoverable through gameplay without explicit explanation

*Phase 3b: Extremification System*
- Flight completion detection (full 10-level runs)
- Modifier intensity scaling after completion
- Revisit detection and extremified modifier application
- Balance testing for extremified difficulty
**Deliverable**: Completing levels 1-10 then revisiting shows intensified modifiers creating "new game plus" difficulty

*Phase 3c: Generation Integration*
- Algorithm response to applied modifiers
- Environmental storytelling through generation choices
- Modifier-influenced enemy spawning patterns
- Architectural changes based on modifier types
**Deliverable**: Level generation algorithms visibly respond to applied modifiers, creating emergent environmental combinations

*Phase 3d: Advanced Sanctuary Features*
- Chest outcome preview system
- Portfolio management interface for greater sanctuaries
- Investment history tracking
- Risk/reward calculation display
**Deliverable**: Players can preview chest contents and see risk/reward calculations before spending currency on modifications

**Phase 4: Polish & Balance**

*Phase 4a: Economic Tuning*
- Currency scarcity curves across depth ranges
- Investment decision pressure points
- Emergency vs strategic spending balance
- Currency sink mechanisms
**Deliverable**: Balanced currency economy where decisions feel meaningful at all depths, with appropriate scarcity pressure

*Phase 4b: Reward System Balance*
- Room clearing reward scaling
- Perfect clearing multiplier tuning
- Abandoned room penalty implementation
- Extraction bonus calculation refinement
**Deliverable**: Reward scaling encourages thorough exploration while maintaining extraction tension throughout depth ranges

*Phase 4c: Discovery Mechanics*
- Hidden system revelation through gameplay
- Community knowledge sharing support
- "Did you know..." moment engineering
- Tutorial-free learning curve optimization
**Deliverable**: Systems can be discovered organically through play, creating shareable "aha!" moments without explicit tutorials

*Phase 4d: Performance & Polish*
- Level state caching optimization
- Memory management for persistent data
- UI/UX polish for all sanctuary interfaces
- Audio/visual feedback for modifier systems
**Deliverable**: Smooth performance with polished UI/UX for all systems, ready for player testing and feedback

---

## Success Indicators

**Discovery**: Players naturally discover sanctuary abandonment through gameplay; community develops extraction terminology; "Did you know..." moments drive engagement.

**Engagement**: First extraction feels like genuine victory; inventory dilemmas create memorable decisions; community shares extraction strategies and failure stories as learning tools.

**Cultural Impact**: "Extract or perish" becomes recognized design philosophy; players create content showcasing modifier discoveries and unique generation combinations; the game influences other designers' risk/reward approaches and procedural variety systems.

---

*\"The surface victory is just permission to begin.\"*

The Cathedral awaits your return - not as mere shelter, but as the sacred destination that gives meaning to every descent into the depths.
