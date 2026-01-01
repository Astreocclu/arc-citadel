//! Integration tests for hierarchical polity system

use std::collections::HashMap;

use arc_citadel::core::types::{PolityId, RulerId, PolityTier, GovernmentType, Species, LocationId};
use arc_citadel::aggregate::polity::{Polity, PolityType, CulturalDrift, SpeciesState, HumanState};
use arc_citadel::aggregate::ruler::{Ruler, PersonalityTrait, Skills, Family, Opinion};
use arc_citadel::aggregate::hierarchy::{get_sovereign, get_vassals, get_all_vassals, is_vassal_of, same_realm};
use arc_citadel::campaign::Location;

fn create_test_hierarchy() -> (HashMap<PolityId, Polity>, HashMap<RulerId, Ruler>) {
    // Create: Empire(1) -> Kingdom(2) -> Duchy(3)
    //                   -> Kingdom(4)

    let mut polities = HashMap::new();
    let mut rulers = HashMap::new();

    // Empire
    let emperor = Ruler::new(
        RulerId(1),
        "Emperor Magnus".to_string(),
        Species::Human,
        55,
        vec![PersonalityTrait::Ambitious, PersonalityTrait::Charismatic],
        Skills::new(7, 5, 6, 3),
        Family::founder(1),
    );
    rulers.insert(emperor.id, emperor);

    polities.insert(PolityId(1), Polity {
        id: PolityId(1),
        name: "Empire of Aldoria".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Empire,
        government: GovernmentType::Autocracy,
        parent: None,
        rulers: vec![RulerId(1)],
        council_roles: HashMap::new(),
        capital: 0,
        population: 100000,
        military_strength: 1000.0,
        economic_strength: 1000.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    // Kingdom under Empire
    let king = Ruler::new(
        RulerId(2),
        "King Aldric".to_string(),
        Species::Human,
        42,
        vec![PersonalityTrait::Honorable, PersonalityTrait::Cautious],
        Skills::new(5, 8, 4, 2),
        Family::founder(2),
    );
    rulers.insert(king.id, king);

    polities.insert(PolityId(2), Polity {
        id: PolityId(2),
        name: "Kingdom of Valheim".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: Some(PolityId(1)),
        rulers: vec![RulerId(2)],
        council_roles: HashMap::new(),
        capital: 1,
        population: 50000,
        military_strength: 500.0,
        economic_strength: 500.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    // Duchy under Kingdom
    let duke = Ruler::new(
        RulerId(3),
        "Duke Rodric".to_string(),
        Species::Human,
        35,
        vec![PersonalityTrait::Warlike],
        Skills::new(3, 9, 2, 1),
        Family::founder(3),
    );
    rulers.insert(duke.id, duke);

    polities.insert(PolityId(3), Polity {
        id: PolityId(3),
        name: "Duchy of Ironhold".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Duchy,
        government: GovernmentType::Autocracy,
        parent: Some(PolityId(2)),
        rulers: vec![RulerId(3)],
        council_roles: HashMap::new(),
        capital: 2,
        population: 20000,
        military_strength: 200.0,
        economic_strength: 200.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    // Another Kingdom under Empire
    polities.insert(PolityId(4), Polity {
        id: PolityId(4),
        name: "Kingdom of Eastmarch".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: Some(PolityId(1)),
        rulers: vec![RulerId(4)],
        council_roles: HashMap::new(),
        capital: 3,
        population: 40000,
        military_strength: 400.0,
        economic_strength: 400.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    (polities, rulers)
}

#[test]
fn test_hierarchy_queries() {
    let (polities, _rulers) = create_test_hierarchy();

    // All should trace to Empire
    assert_eq!(get_sovereign(PolityId(1), &polities), Some(PolityId(1)));
    assert_eq!(get_sovereign(PolityId(2), &polities), Some(PolityId(1)));
    assert_eq!(get_sovereign(PolityId(3), &polities), Some(PolityId(1)));
    assert_eq!(get_sovereign(PolityId(4), &polities), Some(PolityId(1)));

    // Direct vassals of Empire
    let empire_vassals = get_vassals(PolityId(1), &polities);
    assert_eq!(empire_vassals.len(), 2);

    // All vassals of Empire
    let all_empire_vassals = get_all_vassals(PolityId(1), &polities);
    assert_eq!(all_empire_vassals.len(), 3);

    // Vassal relationships
    assert!(is_vassal_of(PolityId(3), PolityId(1), &polities)); // Duchy vassal of Empire
    assert!(is_vassal_of(PolityId(3), PolityId(2), &polities)); // Duchy vassal of Kingdom
    assert!(!is_vassal_of(PolityId(2), PolityId(3), &polities)); // Kingdom not vassal of Duchy

    // Same realm
    assert!(same_realm(PolityId(2), PolityId(4), &polities)); // Both under Empire
    assert!(same_realm(PolityId(3), PolityId(4), &polities)); // Duchy and other Kingdom
}

#[test]
fn test_ruler_opinions() {
    let (_polities, mut rulers) = create_test_hierarchy();

    // Emperor forms opinions of vassal kingdoms
    let emperor = rulers.get_mut(&RulerId(1)).unwrap();
    emperor.set_opinion(PolityId(2), Opinion::new(50));  // Likes Kingdom
    emperor.set_opinion(PolityId(4), Opinion::new(-20)); // Dislikes other Kingdom

    assert_eq!(emperor.get_opinion(PolityId(2)).unwrap().effective_value(), 50);
    assert_eq!(emperor.get_opinion(PolityId(4)).unwrap().effective_value(), -20);

    // King forms opinion of liege
    let king = rulers.get_mut(&RulerId(2)).unwrap();
    king.set_opinion(PolityId(1), Opinion::new(30)); // Respects Emperor

    // War modifier from personalities
    let duke = rulers.get(&RulerId(3)).unwrap();
    assert!(duke.war_modifier() > 0); // Warlike trait increases war likelihood
}

#[test]
fn test_location_controller() {
    let mut castle = Location::new(LocationId(1), "Ironhold Castle".to_string());

    // Initially uncontrolled
    assert!(castle.controller.is_none());

    // Transfer to duchy
    castle.transfer_control(Some(PolityId(3)));
    assert!(castle.is_controlled_by(PolityId(3)));

    // Conquered by kingdom
    castle.transfer_control(Some(PolityId(2)));
    assert!(!castle.is_controlled_by(PolityId(3)));
    assert!(castle.is_controlled_by(PolityId(2)));
}
