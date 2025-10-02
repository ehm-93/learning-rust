**Tileset-Based Procedural Generation System**

**Overview**
Pre-generate a library of chunk templates offline, then assemble them at runtime using connector-based constraints to form coherent paths through infinite space.

**Template Structure**
Each template is a fixed-size tile grid (e.g., 64x64) with metadata:
- Tile data: boolean grid representing walkable/solid
- Connectors: list of openings at edges, each tagged with:
  - Edge side (north/south/east/west)
  - Position along edge (0-63)
  - Path type (main/side/secret or numeric ID)

**Template Generation (Offline)**
1. Run procedural algorithms (DLA, CA, simplex) to generate candidate chunks
2. Analyze each chunk to detect openings at edges
3. Assign path types to connectors based on internal connectivity
4. Validate: ensure internal connectivity between connectors of same path
5. Save valid templates to library (aim for 1000+ templates)

**Runtime Chunk Generation**
1. Hash chunk coordinates to get deterministic seed
2. Query already-generated neighbor chunks for their connectors
3. Filter template library to only those satisfying neighbor constraints:
   - Matching opposite sides (north needs south, etc.)
   - Aligned positions
   - Matching path types
4. Select random template from filtered set using seeded RNG
5. Place template at chunk position

**Constraint Satisfaction**
Two connectors match if:
- Sides are opposite (north ↔ south, east ↔ west)
- Positions align exactly
- Path types match (main connects to main, etc.)

**Path Networks**
Multiple path types create natural topology:
- Main path: primary progression route
- Side paths: optional branches, shortcuts
- Secret paths: hidden connections, rare
Different paths can intersect within templates but only connect at matching typed connectors.

**Benefits**
- Deterministic: same coordinates always generate same chunk
- Infinite: works for any coordinates
- Fast: just template lookup and placement
- Quality: curated templates ensure interesting spaces
- Flexible: path types create intentional structure
