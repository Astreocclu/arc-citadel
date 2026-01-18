# Playability Threshold Spec

> Defines when human playtesting becomes valuable vs. when AI can iterate autonomously.

## Current State Assessment

**What works:**
- Humans select actions based on needs/values (100/100 playtest score)
- Cross-species combat resolution (humans can kill orcs)
- Basic needs system (food, rest, social, safety, purpose)
- Perception and thought generation
- 9 action types with emergent variety

**What's missing (obvious grunt work):**
- Orcs don't fight back (no AI for non-human entities)
- Only 2 species implemented (humans, orcs)
- No equipment system beyond hardcoded weapons
- Combat is one-sided (orcs just stand there)
- No win/loss conditions
- No campaign/strategic layer
- No UI beyond debug terminal

## Playability Threshold Criteria

Human playtesting becomes valuable when ALL of these are met:

### Tier 1: Basic Combat Loop (AI can do this)
- [ ] All combat-capable species fight back when attacked
- [ ] All combat-capable species have appropriate weapons/armor
- [ ] Entities can die from combat (both sides)
- [ ] Combat produces meaningful tactical decisions (positioning, target selection)

### Tier 2: Species Variety (AI can do this)
- [ ] At least 4 species with distinct behaviors (human, orc, dwarf, elf)
- [ ] Species have different values/needs that produce different emergent behavior
- [ ] Species-appropriate equipment and combat stats

### Tier 3: Win/Loss Conditions (AI can do this)
- [ ] Clear victory/defeat states
- [ ] Session can end meaningfully
- [ ] Progress is trackable

### Tier 4: Playable Interface (AI can mostly do this)
- [ ] Entity selection and command issuing works smoothly
- [ ] Can observe what's happening without reading logs
- [ ] Camera controls and zoom
- [ ] Basic feedback on actions (success/failure)

### Tier 5: Strategic Depth (HUMAN INPUT NEEDED)
- [ ] Multiple valid strategies exist
- [ ] Choices feel meaningful
- [ ] Pacing feels right
- [ ] "One more turn" engagement

## When to Involve Human

**AI handles autonomously:**
- Implementing missing systems (combat AI, equipment, species)
- Fixing bugs and edge cases
- Balancing obvious issues (orcs too weak, humans too strong)
- Performance optimization
- Code architecture decisions

**Human input valuable for:**
- "Does this feel fun?" questions
- Pacing and engagement tuning
- Strategic depth evaluation
- UI/UX polish decisions
- Flavor and theming choices
- "What should happen when X?" design questions

## Recommended AI Sprint Before Human Testing

1. **Make orcs fight back** - Add action selection for orc archetype
2. **Equip orcs with weapons** - Axes/clubs so they can hurt humans
3. **Add orc-initiated combat** - Orcs attack nearby humans
4. **Bidirectional combat in playtest** - Track human casualties too
5. **Add 2 more species** - Dwarves and elves with distinct behaviors
6. **Basic win condition** - "Survive 1000 ticks" or "eliminate all hostiles"
7. **UI improvements** - See entities fighting, deaths, etc.

## Success Metric

When the automated playtest shows:
- Casualties on BOTH sides
- Multiple species interacting
- Clear session progression toward win/loss
- Variety in entity behaviors by species

Then human playtesting adds value.

## Current Blocker

**Orcs don't have action selection.** They spawn, stand still, get attacked, and die. This is the single biggest gap before meaningful gameplay exists.

Priority: Implement `select_action_for_orc()` in tick.rs or create an OrcAI system.
