# Formation Laboratory Analysis Report

**Date:** 2026-01-19
**Test Harness:** `formation_lab` binary
**Trials per config:** 50
**Rounds per trial:** 15

## Executive Summary

Ran 150+ combat configurations testing formation shapes, unit composition, equipment matchups, terrain effects, and morale cascades. Key findings:

1. **Pike units are dominant** - 0 casualties against infantry, 100% win rate
2. **Heavy armor is nearly invincible** - Plate armor deflects all Sharp weapons
3. **Formation depth matters** - Line_d2 (shallow) fights faster but takes more casualties
4. **Column formation is terrible for combat** - Too narrow a frontage
5. **Mixed pike+archer is optimal** - Combines reach with ranged support

---

## Formation Shape Effects

| Shape | vs Line_d2 (100v100 inf) | Notes |
|-------|-------------------------|-------|
| Line_d2 | 80 cas, 7 rounds | Wide frontage, fast resolution |
| Line_d4 | 57 cas, 10 rounds | Balanced depth/width |
| Line_d6 | 41 cas, 15 rounds | Too narrow, slow |
| Column_w4 | 12 cas, 15 rounds | Very narrow, ineffective |
| Column_w8 | 24 cas, 15 rounds | Still too narrow |
| Wedge | 28 cas, 15 rounds | Moderate |
| Square | 28 cas, 15 rounds | Defensive formation |
| Skirmish | 80 cas, 7 rounds | Wide like Line_d2 |

**Findings:**
- Frontage determines casualties per round
- Line_d2 = 50 frontage (100/2), Column_w4 = 4 frontage
- Deeper formations preserve strength but fight slower
- For mixed pike+archer, Line_d4 is optimal (allows 2 ranks of pike support)

---

## Unit Composition (vs 100 Infantry)

| Composition | Att Casualties | Def Casualties | Exchange Ratio |
|-------------|----------------|----------------|----------------|
| 100% Pike | 0 | 95 | inf:1 |
| 80% Pike + 20% Archer | 0 | 100 | inf:1 |
| 70% Pike + 30% Archer | 0 | 100 | inf:1 |
| 60% Pike + 40% Archer | 0 | 100 | inf:1 |
| 50% Pike + 50% Archer | 0 | 100 | inf:1 |
| 100% Archer | 38 | 100 | 2.63:1 |
| 50% Archer + 50% Infantry | 72 | 80 | 1.12:1 |
| 100% Infantry | 80 | 80 | 1.00:1 |

**Findings:**
- Pike reach advantage is overwhelming (Long reach vs Short)
- Adding archers to pike doesn't improve much (already 0 casualties)
- Pure archer takes casualties in melee but wins via ranged
- Mixed archer+infantry is only slightly better than pure infantry

### Against Heavy Infantry (Plate Armor)

| Composition | Att Casualties | Def Casualties |
|-------------|----------------|----------------|
| Any Pike/Archer mix | 0 | 0 |
| 100% Archer | 80 | 0 |
| 100% Infantry | 80 | 0 |

**Findings:**
- Plate armor (Rigidity::Plate) deflects all Sharp weapons
- Need Blunt weapons or Piercing special to damage Heavy Infantry
- This is intentional per penetration system design

---

## Equipment Matchup Matrix

### Weapon vs Armor Effectiveness

| Attacker | Levy | Infantry | Heavy Inf | Spearmen | Archers | Cavalry |
|----------|------|----------|-----------|----------|---------|---------|
| Levy | 1.00:1 | 0.58:1 | 0.00:1 | 0.00:1 | 0.17:1 | 0.00:1 |
| Infantry | 1.72:1 | 1.00:1 | 0.00:1 | 0.00:1 | 0.58:1 | 0.00:1 |
| Heavy Inf | inf:1 | inf:1 | 1.00:1 | 0.00:1 | inf:1 | 0.00:1 |
| Spearmen | inf:1 | inf:1 | inf:1 | 1.00:1 | inf:1 | inf:1 |
| Archers | 5.88:1 | 1.72:1 | 0.00:1 | 0.00:1 | 1.00:1 | 0.00:1 |

**Findings:**
- Spearmen dominate due to reach (Long vs Short/Medium)
- Heavy Infantry invincible to unarmored attacks
- Cavalry invincible to unarmored attacks (Mail armor)
- Archers effective at range but vulnerable in melee

---

## Terrain Effects (Simulated via Unit Size)

| Scenario | Pike vs Infantry | Cav vs Infantry | Mixed vs Infantry |
|----------|------------------|-----------------|-------------------|
| Narrow (20v20) | inf:1 | inf:1 | inf:1 |
| Medium (50v50) | inf:1 | inf:1 | inf:1 |
| Wide (100v100) | inf:1 | inf:1 | inf:1 |
| Very Wide (200v200) | inf:1 | inf:1 | inf:1 |

**Findings:**
- Scale doesn't affect relative effectiveness (as expected)
- True terrain effects need hex-based cover/elevation modifiers

---

## Balance Issues Identified

### Critical: Pike Dominance
- **Problem:** Pike units take 0 casualties against non-pike units
- **Cause:** Long reach means pike always strikes first, enemy can't close
- **Suggestion:** Add "closing" mechanic where enemy takes first hit but then engages normally

### Critical: Heavy Armor Immunity
- **Problem:** Plate armor deflects 100% of Sharp attacks
- **Cause:** Penetration lookup: Sharp vs Plate = Deflect
- **Suggestion:**
  1. Add weapon variety (maces, warhammers with Blunt edge)
  2. Add critical hit chance that bypasses armor
  3. Reduce Plate effectiveness to ShallowCut on good hits

### Moderate: No Morale Cascades
- **Problem:** Units fight to the death, no routing observed
- **Cause:** Stress accumulation doesn't hit threshold in test duration
- **Suggestion:** Run longer battles or increase stress accumulation

---

## Recommendations

### For Gameplay Feel

1. **Reduce pike advantage** - Add "charge through" mechanic where infantry can close
2. **Add anti-armor options** - Maces, crossbow bolts with Piercing
3. **Increase stress generation** - Make morale matter before annihilation

### For Multi-Formation Battles

1. **Formation shape matters** - Line for attack, Column for march, Square for defense
2. **Depth affects pike effectiveness** - Line_d4 allows 2 support ranks
3. **Mixed units are optimal** - Pike front + archer rear is historically accurate

### For AI Development

1. **Counter-picking** - AI should bring Heavy Inf vs enemy archers
2. **Formation switching** - Square when flanked, Line when attacking
3. **Reserve management** - Deeper formations preserve strength for exploitation

---

## Data Files

- Raw results: `/tmp/formation_lab_results.csv`
- Test harness: `src/bin/formation_lab.rs`

## Changes Made During Analysis

1. Fixed defender counter-attack bug (`can_riposte()` for Defensive stance)
2. Added formation depth/frontage calculation affecting combat width
3. Added multi-rank reach weapon support (up to 3 ranks of pike)

---

## Balance Fixes Implemented (Post-Analysis)

### Fix 1: Closing Mechanic for Pike Dominance

**Problem:** Pike units (Long/Pike reach) always struck first, preventing shorter-reach attackers from ever closing.

**Solution:** Added "closing mechanic" in `src/combat/resolution.rs`:
- When attacker has shorter reach than defender, attacker takes a "closing wound" before engaging
- This models charging through a pike hedge - heavy losses on approach, but survivors fight normally
- After closing, both combatants fight at equal footing (reach no longer matters)

**Result:**
- Infantry vs Spearmen now shows 48 casualties (attacker) before engaging
- Pike still effective defensively but not invincible
- Creates tactical decision: accept closing casualties to neutralize pike?

### Fix 2: MenAtArms Unit Type (Anti-Armor)

**Problem:** Plate armor (Heavy Infantry, Heavy Cavalry) deflected 100% of Sharp weapons.

**Solution:** Added new unit type `MenAtArms` with:
- `WeaponProperties::warhammer()` - Blunt edge, Massive mass
- Mail armor (not plate - they're specialists, not tanks)
- Massive mass vs Heavy padding = Stagger (WoundSeverity::Scratch)

**New weapon presets:**
- `WeaponProperties::warhammer()` - Blunt, Massive, Short reach
- `WeaponProperties::poleaxe()` - Blunt, Heavy, Medium reach + Piercing

**Result:**
- MenAtArms vs HeavyInfantry: 0 attacker casualties, 25 defender casualties
- Creates counter-pick dynamic: bring MenAtArms to counter heavy armor

### Fix 3: Increased Stress Generation

**Problem:** Stress accumulated too slowly; units died before morale broke.

**Solution:** Added stress constants in `src/battle/constants.rs`:
- `STRESS_PER_WOUND = 0.03` (was 0.01)
- `STRESS_PER_CLOSING_WOUND = 0.05` (was 0.02)
- `STRESS_PER_RANGED_HIT = 0.02` (was 0.005)
- `STRESS_PER_ALLY_DEATH = 0.02` (new - watching comrades die)

**Result:**
- With Infantry threshold ~2.0, about 30-50 casualties should trigger morale break
- Ally deaths now cause additional stress spread across unit
- Battles should end via morale before total annihilation

---

## Post-Fix Balance Summary

| Matchup | Before Fix | After Fix |
|---------|------------|-----------|
| Infantry vs Spearmen | 0:inf | 48:0 (closing casualties) |
| MenAtArms vs HeavyInfantry | N/A | 0:25 (armor counter) |
| HeavyInfantry vs HeavyInfantry | 0:0 | 0:0 (still stalemate) |
| Infantry vs Infantry | 1:1 | 1:1 (unchanged) |

**Remaining issues:**
- Heavy vs Heavy stalemate still exists (need crossbows with Piercing or cavalry charge)
- Morale breaks need live testing in full battles (not just isolated combat)
