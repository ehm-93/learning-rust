# Mobile Computer

Ruggedized smartphone-like device that pairs with all your equipment, authenticates your identity, and provides critical information. Your primary UI and proof of legal existence.

## Core Concept

**Form Factor**: Slightly larger and thicker than a modern smartphone (~15cm × 8cm × 2cm). Touchscreen with physical backup buttons. Built to survive 165 years.

**Identity Chip**: Removable cryptographic ID (like a SIM card) - your legal personhood. Can be transplanted to replacement MC if device breaks.

**Equipment Hub**: Pairs with weapons, tools, suit systems, containers. If you're carrying it, the MC manages it.

**Network**: Wireless connection to colony infrastructure (doors, terminals, trade) and mesh networking to other MCs.

## Diegetic Interface

The MC exists in the game world - you hold it, others can see you using it.

**Access**: Quick-tap raises MC (wrist or hand-held). Time slows to 10% but doesn't pause. You're vulnerable - can't shoot, move slowly.

**Display**:
- **Wrist Mount**: Quick glance, limited interaction
- **Hand-held**: Full access, both hands occupied
- **HUD Projection**: AR glasses/helmet overlays (if equipped)

**Physical States**:
- Visible in darkness (light source)
- Can be damaged (cracked screen, broken buttons, water damage)
- Identity chip removable/transferable

**Functions**:
- Equipment management and status
- Life support monitoring (oxygen, temperature, power)
- Environmental scanning (radiation, C-7, atmosphere, anomalies)
- Map and navigation (acquired data only, no auto-fill)
- Communication (text messages, audio logs, recordings)

**Zone Navigation**:
- Map data from vendors, salvage, or mapping missions
- Current position visible (low power infrastructure beacons still running on integral RTGs?)
- Manual waypoints on known areas
- Physical compass optional (takes equipment slot)
- No minimap - must open MC

## UI Tabs

**Status**: Health, injuries, C-7 exposure, hunger/thirst/fatigue, suit integrity, status effects

**Inventory**: Grid-based (weight/space limited), equipment slots, consumables, drag-and-drop, item inspection

**Map**: Purchased/salvaged data only, varying quality, player waypoints, discovered POIs, faction territories, NPC positions

**Intel**: NPC messages, audio logs, terminal data, quest info, recordings

**Journal**: Manual notes, observations, theories, research tracking

**Relations**: Faction standings, NPC relationships, reputation changes, access privileges

## Colony Integration

**Doors**: Auto-authenticates on approach. Green (granted), Yellow (restricted), Red (locked). Some hackable (leaves traces).

**Terminals**: Wireless or cable (high security applications). Extract data, modify systems.

**Trade**: Automated vendors authenticate via MC. Currency depends on vendor loyalty. Pricing varies by personal/faction rep. Black market requires jailbroken firmware.

**Information Gathering**:
- Passive logging of locations and encounters
- Active recording of conversations
- Network scanning (nearby MCs, security, equipment)
- Data extraction from terminals
- Map acquisition (vendors, salvage, surveys)

## Equipment & Life Support

**Paired Equipment**: Weapons (ammo, condition, jams), tools (battery, settings), containers (inventory). Identity-locked - thieves need techs to strip pairing.

**Power**: MC has minimal internal RTG. Suit systems (O2, heating/cooling) and active equipment drain suit battery. Can disable systems to conserve.

**Life Support Monitoring**:
- Safe zones: Environmental status display, warnings only
- Abandoned zones: Oxygen timer, temperature warnings, power distribution, breach alerts, C-7 accumulation

## Modifications

**Hardware** (technician required): Extended battery, reinforced case, enhanced antenna, encrypted storage

**Software** (technician + faction standing): Privacy firmware (illegal in Corporate zones), black market protocols, enhanced scanners, auto-management AI

**Exploits** (specialists, faction-specific): Backdoors (leaves traces), surveillance bypass, authentication spoof (very illegal), data wipe

**Tradeoffs**: Privacy mods suspicious in Corporate zones, hacking tools attract security, black market ties to criminal networks, mods increase instability

## Whispers Corruption

**0-20%**: Normal operation

**20-50%**: UI glitches, phantom map markers, strange log entries, unsolicited waypoints (accurate), early environmental warnings

**50-80%**: Mystery recordings, reveals NPC locations, security codes appear, advice messages, frequent anomalies

**80%+**: Autonomous behavior, impossible information, direct entity communication, constant corruption

Creates paranoia - info is inconsistently accurate, NPCs notice weird behavior, security gets suspicious.

## Implementation Notes

**UI**: MC screen as 3D object, minimal HUD, 10% time dilation, camera focus/blur

**Audio**: Device sounds (beeps, clicks), warning tones, Whisper glitches, background audio continues

**UX**: Quick actions without full open, hotkeys for tabs, radial menus for consumables, smart defaults

**Progression**: Mods unlock via faction rep, better techs in different settlements, rare firmware in zones

## Optional Advanced Mechanics

**Processing Power**: Limited CPU/memory, advanced equipment needs upgrades, creates equipment progression

**Power Draw**: More powerful processors drain suit battery faster, trade-off between capability and expedition duration

**Hacking**: Install modules for doors/terminals/cameras/turrets, failed hacks trigger alarms and traces, stealth gameplay

**Identity Spoofing**: Extremely illegal, temporary cloning of nearby MC, very short duration, high detection risk, severe consequences. Automatic "impossible travel" detection? NPC ID activates two doors 1km away from each other = instant lockdown?

**Equipment Plugins**: Certain gear requires specific modules (Military, Medical, Engineering, Scientific, Faction DRM [multiple levels?]). Can't carry all simultaneously, faction-specific, reputation-gated. Questions: Plugin bays limited by MC tier? CPU/memory requirements?

**Data Storage**: Finite 2TB, logs/maps/intel consume space, forces deletion choices, external storage option, Old Company maps are huge

**Network Quality**: Antenna affects range, dead zones mesh-only, environmental interference, upgrades improve extraction range

**Maintenance**: Degrades over time, physical damage, requires tech repairs, rare catastrophic failure. Identity chip salvageable from destroyed MC. Replacement MCs scarce/valuable. Losing chip = permanent identity loss.

**Multi-MC**: Backup with different config, hot-swap profiles, chip goes in one at a time, swap risk, expensive/heavy

## Quick Reference

**Core**: Inventory, life support, map/nav (purchased data only), intel storage, faction relations, door auth, terminal access, info gathering

**Physical**: 3D object, 10% time dilation, light source, damageable, needs maintenance, C-7 corrupts over time

**Design Goals**: Natural player extension, active info gathering gameplay, risk/consequence in danger, progressive corruption horror, faction customization paths

## Reference

- Stalker PDA (diegetic pause, time dilation, info gathering)
- Fallout Pip-Boy (equipment hub, status tracking)
- Dead Space RIG (diegetic UI, health/resource display)
- Metro 2033 wrist computer (physical device, light source risk)
- Prey (2017) TranScribe (audio logs, environmental scanning)
- Subnautica PDA (survival data, crafting blueprints)
