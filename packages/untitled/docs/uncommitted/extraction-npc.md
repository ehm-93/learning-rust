# Extraction NPC Service Specification

## Overview
Players can hire an extraction specialist before diving into the dungeon. The specialist waits at a designated Lesser Sanctuary to provide one-time full inventory extraction. Payment is made upfront and non-refundable.

---

## Core Mechanics

### Hiring Process
**Location:** Cathedral hub only

**Steps:**
1. Player approaches Extraction Coordinator NPC
2. Selects target Lesser Sanctuary level from available options
3. Views cost in Extraction Seals
4. Confirms purchase - currency immediately deducted
5. Contract is active for next dive only

### Pricing Structure
Cost varies by depth tier and distance to next Greater Sanctuary. Deeper tiers require currencies that only drop at those depths, creating natural progression gates.

**Tier 1 (Levels 1-29):**
- Level 11: 9 Extraction Seals
- Level 15: 5 Extraction Seals
- Level 19: 1 Extraction Seal
- Level 20: N/A (Greater Sanctuary - free full extraction)

**Tier 2 (Levels 30-59):**
- Level 31: 9 Seals + 5 Abyssal Marks
- Level 35: 5 Seals + 3 Abyssal Marks
- Level 39: 1 Seal + 1 Abyssal Mark
- Level 40: N/A (Greater Sanctuary - free full extraction)

**Tier 3 (Levels 60-89):**
- Level 61: 9 Seals + 5 Abyssal Marks + 10 Void Essence
- Level 65: 5 Seals + 3 Abyssal Marks + 6 Void Essence
- Level 69: 1 Seal + 1 Abyssal Mark + 2 Void Essence
- Level 70: N/A (Greater Sanctuary - free full extraction)

**Tier 4 (Levels 90+):**
Same pattern continues with all three currencies at scaled costs.

**Formula Logic:**
- Base Seals cost decreases with proximity to next Greater Sanctuary
- Tier-specific currencies gate access to deep-tier contracts
- High costs near insertion points discourage trivial farming
- Must actually play at depth X to safely farm depth X

### Contract Activation
**When reaching contracted level:**
- Lesser Sanctuary displays standard partial extraction option
- Additional "Full Extraction (Hired)" option appears
- Functions like Greater Sanctuary extraction: all inventory kept + 1 bonus chest
- Contract consumed on use

**If bypassing contracted level:**
- Contract remains unused but cannot be refunded
- Sunk cost - player overcommitted

**If dying before reaching contracted level:**
- Contract and currency lost with run
- No compensation

---

## Currency: Multi-Tier Extraction Economics

### Currency Types

**Extraction Seals** (Universal)
- Drops at all levels
- Base cost for all contracts
- Primary source: Greater Sanctuary chests (1-3 per extraction)
- Secondary sources: Elite enemies, bosses, quests

**Abyssal Marks** (Tier 2+)
- Only drops at Levels 30-59
- Required for Level 30-59 contracts
- Drop rate: 2-4 per successful Tier 2 run
- Gates access to deep-tier farming

**Void Essence** (Tier 3+)
- Only drops at Levels 60+
- Required for Level 60+ contracts
- Drop rate: 1-2 per successful Tier 3+ run
- Gates access to extreme-depth farming

### Self-Gating Design

**You cannot hire extraction at a tier without having reached that tier.** Level 35 contracts require Abyssal Marks, which only drop at Level 30+. This creates natural progression:

1. Reach Level 30 for first time (risky, no contract safety net)
2. Collect Abyssal Marks from the run
3. Use those Marks to hire safer Level 30-39 extractions for farming
4. Accumulate enough gear/currency to push to Level 60
5. Repeat cycle with Void Essence

**Target Economy:**
- Tier 1: Every run can be contracted (Seals drop universally)
- Tier 2: Every 2-3 deep runs funds 1 safe farm run
- Tier 3: Every 3-4 extreme runs funds 1 safe farm run

This prevents constant contracted farming at high tiers while making contracts accessible enough for regular strategic use.

---

## Rules & Constraints

### One Contract Per Run
Only one active contract allowed per dive. Cannot purchase multiple safety nets across different levels.

### No Refunds
Once purchased, contracts cannot be cancelled or refunded. Commitment is permanent.

### Run-Specific
Contracts do not carry between runs. Each dive requires a fresh contract if desired.

### Emergency Extraction Incompatibility
Contracted extraction only available at the specified Lesser Sanctuary. Does not apply to emergency extractions triggered elsewhere.

---

## Strategic Use Cases

### Optimal Applications
- **Material farming:** Contract mid-depth (Level 25) to safely gather specific resources
- **Quest completion:** Guarantee extraction with valuable quest items
- **Build testing:** Practice at specific depth ranges with safety net
- **Lucky finds:** Secure extraction after unexpected legendary drops

### Inefficient Applications
- **Over-contracting:** Buying Level 21 extraction when confident in reaching Level 30
- **Under-pushing:** Contracting Level 29 instead of pushing one more level to free Greater Sanctuary
- **Panic buying:** Spending Seals reactively instead of planning strategically

### Expected Player Archetypes
- **Speedrunners:** Never use contracts, consider them wasteful
- **Farmers:** Regular users at mid-range levels for efficient material gathering
- **Quest focused:** Strategic users for specific objectives only
- **Risk-averse:** Over-users who learn efficiency through experience

---

## User Interface

### Cathedral Contract Screen
- List of available Lesser Sanctuaries based on highest Greater Sanctuary reached
- Cost display per level with visual price curve
- Current Extraction Seal balance
- Confirmation dialog: "This specialist will wait at Level X. Payment is non-refundable. Proceed?"

### In-Dungeon Sanctuary UI
**Lesser Sanctuaries:**
- Standard option: "Partial Extraction" (equipped gear + limited inventory)
- If contracted: "Full Extraction (Hired)" option highlighted/glowing
- Clear visual indicator of contracted service availability

**Greater Sanctuaries:**
- Standard option: "Full Extraction" (always available, no contract needed)
- No contract options displayed

---

## Lore Integration

**The Extraction Coordinator** manages a network of specialists willing to descend into Hell for profit. Hiring them means paying a professional to risk entering the depths to extract you.

**Dialog Examples:**
- "Level 23? I'll have someone waiting. They charge whether you show up or not, so don't die."
- "Level 11 already? Expensive for such a short dive, but your coin."
- "Level 29? Bold. You're betting you'll make it that far. Most don't."
- "Changed your mind? Too late - they're already gone. Should've thought harder before paying."

**Narrative Flavor:**
- Extraction specialists are former delvers who know the depths
- They descend from sanctuaries, not the surface
- Payment covers their risk, not your success
- Professional relationship, not heroic rescue

---

## Balance Considerations

### Tuning Levers

**If farming becomes dominant:**
- Increase insertion-adjacent costs (Level 21, 31, etc.)
- Add diminishing returns for frequent contracting
- Reduce Seal drop rates slightly

**If contracts are rarely used:**
- Decrease overall costs
- Increase Seal drop rates
- Add bonus benefits to contracted extraction (better chest rolls, guaranteed rare)

**Currency generation target:** 
Players should accumulate enough Seals to contract 1-in-3 or 1-in-4 runs. Too many = trivializes risk. Too few = feels like trap option.

---

## Future Expansion Possibilities

### Quality Tiers
Different specialist NPCs with varying reliability:
- Veteran (expensive, guaranteed)
- Standard (normal cost)
- Rookie (cheap, small failure chance)

### Modifier Integration
Since modifiers are discovered after contracts are purchased, direct modifier-contract interactions don't work. However, modifiers could affect extraction indirectly:
- "Lesser Sanctuaries provide full extraction naturally" (makes contracts redundant)
- "All sanctuaries are contested - combat required to extract" (makes contracted extraction risky)
- "Inventory capacity doubled" (reduces need for early extraction)

### Enhanced Services
- Multi-level contracts (more expensive, multiple safety nets)
- Priority extraction (faster animation, cosmetic)
- Insurance contracts (partial Seal refund on death)

---

## Design Intent

Provide player agency over extraction risk without removing the core tension. Contracts are safety you can buy, but at significant resource cost. The system creates:

- Strategic pre-planning (not reactive panic buttons)
- Economic depth (Seal management becomes meta-game)
- Meaningful sunk costs (forces commitment to reaching contracted level)
- Build diversity (enables different risk profiles and farming strategies)

The extraction tension remains - players choose between spending currency for safety or risking everything for greater rewards. The choice shifts from "extract or push" to "pay for safety or gamble naturally."
