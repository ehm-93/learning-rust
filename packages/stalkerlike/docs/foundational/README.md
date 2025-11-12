# Foundational Game Documentation

This directory contains the core world-building and game design documents for **Stalkerlike**, a survival horror game set in a dying underground colony on Ross 154a.

## Overview

**Stalkerlike** is a first-person survival horror game inspired by the STALKER series, set in an abandoned underground mining colony 10km beneath the surface of a hostile alien world. You're nobody special - just another worker trying to survive in a dying colony fractured by 110 years of cold civil war.

**Year 165**: Earth went silent 162 years ago (nuclear war, Year 3). The colony that once housed 50,000 now holds only 5,000 survivors. Infrastructure fails, factions war in the shadows, and an exotic material called Catalyst-7 slowly transforms those who venture into the deep abandoned zones.

## Core Documents

### [Setting Overview](./setting.md)
The world of Ross 154a - a hostile metal-rich planetary core orbiting a red dwarf star:
- **The Planet**: Tidally locked iron core with lethal surface radiation
- **The Colony**: 10-20km deep underground settlement, built for 50,000, now 5,000 remain
- **The Isolation**: 165 years since Earth went silent, 162 years since nuclear war
- **Catalyst-7**: The exotic FTL material that now poisons and transforms the colonists
- **The Horror**: Multiple layers of existential dread (the Whispers, the Touched, Dissolution Sites)
- **Temperature Gradient**: Comfortable at surface (0-20°C), rising to dangerous heat in deep zones (80°C at 10km, 140°C at 20km)

### [Timeline](./timeline.md)
Major events in colony history from Year 0 to present (Year 165):
- **Year 3**: Earth's nuclear war - Board suppresses the truth
- **D+13 (Years 13-15)**: 35,000 dead when the truth is revealed
- **Year 47**: The Voltaic Strike - Union radiation attack kills 1,200
- **Years 47-55**: Retaliation campaigns and assassinations
- **Year 92**: University abandons FTL research after 800 deaths
- **Year 110**: Surface Exodus survivors all dead or transformed
- **Year 165**: Present day - slow decline and cold war

### [Core Mechanics](./mechanics.md)
Game systems and survival mechanics:
- **Stalker-Like Foundation**: Nobody special, persistent world, exploration-driven
- **Energy Economy**: Power generation, battery management, limited expedition duration
- **Life Support**: Safe settlements vs abandoned zones requiring full suit systems
- **Environmental Hazards**: Temperature, oxygen, pressure, equipment failure
- **C-7 Exposure**: Accumulating horror - Whispers → mutations → transformation
- **Faction Relations**: Territory control, reputation, proxy conflicts
- **The Stalker Loop**: Prepare → Explore → Survive → Return → Progress

### [Factions](./factions/)
Five major groups in a 110-year cold civil war:
- **[Corporate](./factions/corporate.md)** (1,500): Authoritarian control, dying reactor, maintains the lie
- **[Union](./factions/union.md)** (1,200): Three ideological subfactions in uneasy alliance
- **[Frontier](./factions/frontier.md)** (1,800): Fractured majority, proxy war battleground
- **[Crystalline Faith](./factions/crystalline-faith.md)** (200): Death cult seeking transcendence through C-7
- **[University](./factions/university.md)** (300): Trapped intellectuals, abandoned FTL research

## Game Design Pillars

### 1. Nobody Special
You're just another worker in a colony of 5,000. The world doesn't revolve around you. Conflicts happen whether you're there or not. Your absence is barely noticed.

### 2. Survival Horror Through Scarcity
- Limited energy constrains exploration time
- Life support failures create tension
- C-7 exposure builds up over time
- Every expedition is a calculated risk

### 3. Exploration as Discovery
- No quest markers or handholding
- Environmental storytelling reveals colony history
- Emergent objectives discovered through exploration
- The Zone rewards curiosity and punishes carelessness

### 4. Persistent Consequences
- Faction wars escalate without your intervention
- Systems degrade and fail over time
- Actions have ripple effects across the colony
- The world remembers your choices

### 5. The Creeping Horror
Not jump scares, but existential dread:
- The Whispers providing intel you can't fully trust
- Watching others slowly transform into the Touched
- Dissolution Sites where people simply cease to exist
- The knowledge that Earth is dead and you're stranded
- The slow, patient heat rising from the planet's core

## Getting Started

For implementing game features:
1. Read `setting.md` for world context
2. Review `mechanics.md` for core systems
3. Check faction documents for faction-specific content
4. Reference `timeline.md` for historical events

For editor development:
- This documentation informs what we're building tools to create
- Level design should reflect the underground colony aesthetic
- Faction territories need distinct visual identities
- Temperature gradients affect deep zone design

## Related Documentation

- `../dev/iter-01.md` - Current editor development iteration
- `../../README.md` - Stalkerlike project overview
