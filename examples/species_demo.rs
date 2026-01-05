//! Demo: Species Behavior System in Action

use arc_citadel::aggregate::behavior::{get_behavior, PolityBehavior};
use arc_citadel::aggregate::events::EventType;
use arc_citadel::aggregate::polity::*;
use arc_citadel::aggregate::world::AggregateWorld;
use arc_citadel::core::types::{GovernmentType, PolityId, PolityTier, Species};
use arc_citadel::entity::species::gnoll::GnollValues;
use arc_citadel::rules::value_dynamics::TickDelta;
use arc_citadel::simulation::value_dynamics::apply_tick_dynamics;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║       🐺 SPECIES BEHAVIOR SYSTEM DEMONSTRATION 🐺            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    demo_value_dynamics();
    demo_gnoll_polity();
    demo_vampire_polity();
    demo_kobold_polity();
}

fn demo_value_dynamics() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  📈 VALUE DYNAMICS: Watch a Gnoll's bloodlust rise over time");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut gnoll = GnollValues {
        bloodlust: 0.55,
        pack_instinct: 0.65,
        hunger: 0.50,
        cruelty: 0.40,
        dominance: 0.35,
    };

    // Gnoll bloodlust increases by 0.002 per tick (from gnoll.toml)
    let deltas = vec![
        TickDelta {
            value_name: "bloodlust".to_string(),
            delta: 0.002,
            min: 0.0,
            max: 1.0,
        },
        TickDelta {
            value_name: "hunger".to_string(),
            delta: 0.003,
            min: 0.0,
            max: 1.0,
        },
    ];

    println!("  Initial state:");
    println!(
        "    bloodlust: {:.2}  hunger: {:.2}",
        gnoll.bloodlust, gnoll.hunger
    );
    println!();

    for tick in 1..=50 {
        apply_tick_dynamics(&mut gnoll, &deltas);

        if tick % 10 == 0 {
            let bloodlust_bar = "█".repeat((gnoll.bloodlust * 20.0) as usize);
            let hunger_bar = "█".repeat((gnoll.hunger * 20.0) as usize);
            println!(
                "  Tick {:3}: bloodlust [{:<20}] {:.2}",
                tick, bloodlust_bar, gnoll.bloodlust
            );
            println!(
                "           hunger    [{:<20}] {:.2}",
                hunger_bar, gnoll.hunger
            );

            // Check thresholds
            if gnoll.bloodlust >= 0.6 {
                println!("           ⚔️  BLOODLUST THRESHOLD REACHED - Attack action triggered!");
            }
            if gnoll.hunger >= 0.55 {
                println!("           🍖 HUNGER THRESHOLD REACHED - Gather action triggered!");
            }
            println!();
        }
    }
}

fn demo_gnoll_polity() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  🐺 GNOLL POLITY: The Howling Maw tribe's pack frenzy builds");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut polity = create_gnoll_polity();
    let world = create_test_world();
    let behavior = get_behavior(Species::Gnoll);

    // Start with moderate frenzy
    if let Some(state) = polity.gnoll_state_mut() {
        state.pack_frenzy = 0.5;
        state.demon_taint = 0.3;
    }

    println!("  The Howling Maw tribe stirs...\n");

    // Simulate winning battles
    for round in 1..=3 {
        let state = polity.gnoll_state().unwrap();
        println!(
            "  Round {}: pack_frenzy = {:.2}, demon_taint = {:.2}",
            round, state.pack_frenzy, state.demon_taint
        );

        let events = behavior.tick(&polity, &world, round);
        for event in &events {
            match event {
                EventType::RaidLaunched { .. } => {
                    println!("    → 🔥 RAID LAUNCHED against weak neighbors!")
                }
                EventType::CorruptionSpreads { intensity, .. } => println!(
                    "    → 👿 Demonic corruption spreads (intensity: {:.2})",
                    intensity
                ),
                _ => {}
            }
        }

        // Simulate a battle victory
        let polity_id = polity.id;
        behavior.on_event(
            &mut polity,
            &EventType::BattleWon { polity: polity_id },
            &world,
        );
        println!("    → ⚔️  Battle won! Pack frenzy increases!\n");
    }

    let final_state = polity.gnoll_state().unwrap();
    println!(
        "  Final state: pack_frenzy = {:.2} 🔥🔥🔥",
        final_state.pack_frenzy
    );
    println!();
}

fn demo_vampire_polity() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  🧛 VAMPIRE POLITY: The Crimson Court expands its influence");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut polity = create_vampire_polity();
    let world = create_test_world();
    let behavior = get_behavior(Species::Vampire);

    // Set up initial state
    if let Some(state) = polity.vampire_state_mut() {
        state.blood_debt_owed = 50;
        state.thrall_network = vec![]; // Empty - needs thralls
    }

    println!("  The Crimson Court schemes in the shadows...\n");

    let state = polity.vampire_state().unwrap();
    println!(
        "  Initial: thralls = {:?}, blood_debt = {}",
        state.thrall_network, state.blood_debt_owed
    );

    let events = behavior.tick(&polity, &world, 1);
    for event in &events {
        match event {
            EventType::InfiltrationAttempt { .. } => {
                println!("    → 🕸️  Infiltration attempt - seeking new thralls!")
            }
            EventType::TributeDemanded { amount, .. } => {
                println!("    → 💰 Tribute demanded: {} blood debt owed", amount)
            }
            _ => {}
        }
    }

    // Simulate successful infiltration
    println!("\n  A neighboring noble falls under the Court's sway...");
    let polity_id = polity.id;
    behavior.on_event(
        &mut polity,
        &EventType::InfiltrationSuccess {
            infiltrator: polity_id,
            target: PolityId(99),
        },
        &world,
    );

    let state = polity.vampire_state().unwrap();
    println!(
        "    → 🧛 New thrall acquired! Network: {:?}",
        state.thrall_network
    );

    // Simulate tribute payment
    println!("\n  A debtor pays their blood debt...");
    behavior.on_event(
        &mut polity,
        &EventType::TributePaid {
            to: polity_id,
            amount: 30,
        },
        &world,
    );

    let state = polity.vampire_state().unwrap();
    println!("    → 💸 Debt reduced to: {}", state.blood_debt_owed);
    println!();
}

fn demo_kobold_polity() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  🐲 KOBOLD POLITY: The Deepscale Warren plots revenge");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut polity = create_kobold_polity();
    let world = create_test_world();
    let behavior = get_behavior(Species::Kobold);

    // Set up vengeful kobolds with tunnel network
    if let Some(state) = polity.kobold_state_mut() {
        state.tunnel_network = 12;
        state.trap_density = 0.3;
        state.grudge_targets = vec![42, 77]; // They remember who wronged them!
        state.dragon_worship = 0.5;
    }

    println!("  The Deepscale Warren digs deeper and plots...\n");

    let state = polity.kobold_state().unwrap();
    println!(
        "  Initial: tunnels = {}, trap_density = {:.2}, grudges = {:?}",
        state.tunnel_network, state.trap_density, state.grudge_targets
    );

    let events = behavior.tick(&polity, &world, 1);
    for event in &events {
        match event {
            EventType::TrapConstruction { trap_count, .. } => {
                println!("    → 🪤 Building {} new traps in the tunnels!", trap_count)
            }
            EventType::SpiteRaid { target, .. } => {
                println!("    → 😈 SPITE RAID against enemy #{} - revenge!", target.0)
            }
            EventType::DragonTributeOffered { .. } => {
                println!("    → 🐉 Offering tribute to their dragon master!")
            }
            _ => {}
        }
    }

    // Simulate trap success
    println!("\n  An adventuring party stumbles into the warrens...");
    let polity_id = polity.id;
    behavior.on_event(
        &mut polity,
        &EventType::TrapTriggered {
            polity: polity_id,
            casualties: 3,
        },
        &world,
    );

    let state = polity.kobold_state().unwrap();
    println!("    → 💀 Trap triggered! 3 casualties!");
    println!(
        "    → 📈 Tunnel network expanded to: {}",
        state.tunnel_network
    );

    // Simulate spite raid completion
    println!("\n  The spite raid succeeds...");
    behavior.on_event(
        &mut polity,
        &EventType::SpiteRaid {
            attacker: polity_id,
            target: PolityId(42),
        },
        &world,
    );

    let state = polity.kobold_state().unwrap();
    println!(
        "    → ✓ Grudge against #42 settled! Remaining: {:?}",
        state.grudge_targets
    );
    println!();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  ✨ DEMO COMPLETE - Emergent behavior from simple rules! ✨");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

fn create_gnoll_polity() -> Polity {
    Polity {
        id: PolityId(1),
        name: "The Howling Maw".to_string(),
        species: Species::Gnoll,
        polity_type: PolityType::Horde,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: None,
        rulers: vec![],
        council_roles: HashMap::new(),
        population: 2000,
        capital: 0,
        military_strength: 150.0,
        economic_strength: 50.0,
        founding_conditions: FoundingConditions::default(),
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Gnoll(GnollState::default()),
        alive: true,
    }
}

fn create_vampire_polity() -> Polity {
    Polity {
        id: PolityId(2),
        name: "The Crimson Court".to_string(),
        species: Species::Vampire,
        polity_type: PolityType::Court,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: None,
        rulers: vec![],
        council_roles: HashMap::new(),
        population: 500,
        capital: 0,
        military_strength: 80.0,
        economic_strength: 200.0,
        founding_conditions: FoundingConditions::default(),
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Vampire(VampireState::default()),
        alive: true,
    }
}

fn create_kobold_polity() -> Polity {
    Polity {
        id: PolityId(3),
        name: "The Deepscale Warren".to_string(),
        species: Species::Kobold,
        polity_type: PolityType::Horde,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: None,
        rulers: vec![],
        council_roles: HashMap::new(),
        population: 800,
        capital: 0,
        military_strength: 60.0,
        economic_strength: 40.0,
        founding_conditions: FoundingConditions::default(),
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Kobold(KoboldState::default()),
        alive: true,
    }
}

fn create_test_world() -> AggregateWorld {
    AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42))
}
