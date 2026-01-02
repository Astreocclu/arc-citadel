//! Production recipes - define what buildings produce
//!
//! Recipes specify input resources, output resources, work required,
//! and which building type can execute them.

use serde::{Deserialize, Serialize};
use crate::simulation::resource_zone::ResourceType;
use crate::city::building::BuildingType;

/// A production recipe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Building type that can execute this recipe
    pub building_type: BuildingType,
    /// Input resources consumed
    pub inputs: Vec<(ResourceType, u32)>,
    /// Output resources produced
    pub outputs: Vec<(ResourceType, u32)>,
    /// Work units required to complete one production cycle
    pub work_required: u32,
    /// Workers needed for full speed
    pub workers_needed: u32,
}

impl Recipe {
    /// Calculate production rate based on worker count
    /// Returns multiplier (0.0 to 1.2 max)
    pub fn production_rate(&self, workers: u32) -> f32 {
        if self.workers_needed == 0 {
            return 1.0;
        }
        (workers as f32 / self.workers_needed as f32).min(1.2)
    }
}

/// Catalog of all available recipes
#[derive(Debug, Clone, Default)]
pub struct RecipeCatalog {
    recipes: Vec<Recipe>,
}

impl RecipeCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load default recipes (hardcoded for now)
    pub fn with_defaults() -> Self {
        let mut catalog = Self::new();

        // Farm produces food
        catalog.add(Recipe {
            id: "farm_food".into(),
            name: "Grow Food".into(),
            building_type: BuildingType::Farm,
            inputs: vec![],
            outputs: vec![(ResourceType::Food, 5)],
            work_required: 100,
            workers_needed: 2,
        });

        // Workshop: ore -> iron
        catalog.add(Recipe {
            id: "smelt_iron".into(),
            name: "Smelt Iron".into(),
            building_type: BuildingType::Workshop,
            inputs: vec![(ResourceType::Ore, 3)],
            outputs: vec![(ResourceType::Iron, 1)],
            work_required: 50,
            workers_needed: 1,
        });

        // Workshop: wood -> cloth (using wood as fiber proxy)
        catalog.add(Recipe {
            id: "weave_cloth".into(),
            name: "Weave Cloth".into(),
            building_type: BuildingType::Workshop,
            inputs: vec![(ResourceType::Wood, 2)],
            outputs: vec![(ResourceType::Cloth, 1)],
            work_required: 40,
            workers_needed: 1,
        });

        catalog
    }

    /// Add a recipe to the catalog
    pub fn add(&mut self, recipe: Recipe) {
        self.recipes.push(recipe);
    }

    /// Get a recipe by ID
    pub fn get(&self, id: &str) -> Option<&Recipe> {
        self.recipes.iter().find(|r| r.id == id)
    }

    /// Get all recipes for a specific building type
    pub fn for_building(&self, building_type: BuildingType) -> impl Iterator<Item = &Recipe> {
        self.recipes.iter().filter(move |r| r.building_type == building_type)
    }

    /// Get all recipes
    pub fn all(&self) -> &[Recipe] {
        &self.recipes
    }

    /// Load recipes from a TOML file
    pub fn load_from_toml(path: &std::path::Path) -> Result<Self, RecipeLoadError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RecipeLoadError::IoError(e.to_string()))?;
        Self::parse_toml(&content)
    }

    /// Parse recipes from TOML string
    pub fn parse_toml(content: &str) -> Result<Self, RecipeLoadError> {
        let toml_data: TomlRecipes = toml::from_str(content)
            .map_err(|e| RecipeLoadError::ParseError(e.to_string()))?;

        let mut catalog = Self::new();
        for recipe in toml_data.recipes {
            catalog.add(recipe.into_recipe()?);
        }
        Ok(catalog)
    }
}

/// Error type for recipe loading
#[derive(Debug, Clone)]
pub enum RecipeLoadError {
    IoError(String),
    ParseError(String),
    InvalidBuildingType(String),
    InvalidResourceType(String),
}

impl std::fmt::Display for RecipeLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecipeLoadError::IoError(e) => write!(f, "IO error: {}", e),
            RecipeLoadError::ParseError(e) => write!(f, "Parse error: {}", e),
            RecipeLoadError::InvalidBuildingType(e) => write!(f, "Invalid building type: {}", e),
            RecipeLoadError::InvalidResourceType(e) => write!(f, "Invalid resource type: {}", e),
        }
    }
}

impl std::error::Error for RecipeLoadError {}

/// TOML representation of recipes file
#[derive(Debug, Deserialize)]
struct TomlRecipes {
    recipes: Vec<TomlRecipe>,
}

/// TOML representation of a single recipe
#[derive(Debug, Deserialize)]
struct TomlRecipe {
    id: String,
    name: String,
    building_type: String,
    #[serde(default)]
    inputs: Vec<TomlResourceAmount>,
    outputs: Vec<TomlResourceAmount>,
    work_required: u32,
    workers_needed: u32,
}

/// TOML representation of a resource amount
#[derive(Debug, Deserialize)]
struct TomlResourceAmount {
    resource: String,
    amount: u32,
}

impl TomlRecipe {
    fn into_recipe(self) -> Result<Recipe, RecipeLoadError> {
        let building_type = match self.building_type.to_lowercase().as_str() {
            "house" => BuildingType::House,
            "farm" => BuildingType::Farm,
            "workshop" => BuildingType::Workshop,
            "granary" => BuildingType::Granary,
            "wall" => BuildingType::Wall,
            "gate" => BuildingType::Gate,
            _ => return Err(RecipeLoadError::InvalidBuildingType(self.building_type)),
        };

        let inputs = self.inputs
            .into_iter()
            .map(|ra| ra.into_resource_amount())
            .collect::<Result<Vec<_>, _>>()?;

        let outputs = self.outputs
            .into_iter()
            .map(|ra| ra.into_resource_amount())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Recipe {
            id: self.id,
            name: self.name,
            building_type,
            inputs,
            outputs,
            work_required: self.work_required,
            workers_needed: self.workers_needed,
        })
    }
}

impl TomlResourceAmount {
    fn into_resource_amount(self) -> Result<(ResourceType, u32), RecipeLoadError> {
        let resource = match self.resource.to_lowercase().as_str() {
            "wood" => ResourceType::Wood,
            "stone" => ResourceType::Stone,
            "ore" => ResourceType::Ore,
            "iron" => ResourceType::Iron,
            "cloth" => ResourceType::Cloth,
            "food" => ResourceType::Food,
            _ => return Err(RecipeLoadError::InvalidResourceType(self.resource)),
        };
        Ok((resource, self.amount))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_production_rate() {
        let recipe = Recipe {
            id: "test".into(),
            name: "Test".into(),
            building_type: BuildingType::Farm,
            inputs: vec![],
            outputs: vec![(ResourceType::Food, 1)],
            work_required: 100,
            workers_needed: 2,
        };

        // 0 workers = 0 rate
        assert!((recipe.production_rate(0) - 0.0).abs() < 0.01);
        // 1 worker = 50% rate
        assert!((recipe.production_rate(1) - 0.5).abs() < 0.01);
        // 2 workers = 100% rate
        assert!((recipe.production_rate(2) - 1.0).abs() < 0.01);
        // 3 workers = 120% rate (capped)
        assert!((recipe.production_rate(3) - 1.2).abs() < 0.01);
        // Many workers still capped at 120%
        assert!((recipe.production_rate(10) - 1.2).abs() < 0.01);
    }

    #[test]
    fn test_recipe_production_rate_zero_workers_needed() {
        let recipe = Recipe {
            id: "auto".into(),
            name: "Automated".into(),
            building_type: BuildingType::Granary,
            inputs: vec![],
            outputs: vec![(ResourceType::Food, 1)],
            work_required: 10,
            workers_needed: 0, // No workers needed
        };

        // Should always return 1.0 when workers_needed is 0
        assert!((recipe.production_rate(0) - 1.0).abs() < 0.01);
        assert!((recipe.production_rate(5) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_recipe_catalog_defaults() {
        let catalog = RecipeCatalog::with_defaults();

        let farm_food = catalog.get("farm_food");
        assert!(farm_food.is_some());
        let farm_food = farm_food.unwrap();
        assert_eq!(farm_food.building_type, BuildingType::Farm);
        assert_eq!(farm_food.work_required, 100);
        assert_eq!(farm_food.outputs.len(), 1);
        assert_eq!(farm_food.outputs[0], (ResourceType::Food, 5));

        let smelt = catalog.get("smelt_iron");
        assert!(smelt.is_some());
        let smelt = smelt.unwrap();
        assert_eq!(smelt.building_type, BuildingType::Workshop);
        assert_eq!(smelt.inputs.len(), 1);
        assert_eq!(smelt.inputs[0], (ResourceType::Ore, 3));
    }

    #[test]
    fn test_recipe_catalog_for_building() {
        let catalog = RecipeCatalog::with_defaults();

        let farm_recipes: Vec<_> = catalog.for_building(BuildingType::Farm).collect();
        assert_eq!(farm_recipes.len(), 1);
        assert_eq!(farm_recipes[0].id, "farm_food");

        let workshop_recipes: Vec<_> = catalog.for_building(BuildingType::Workshop).collect();
        assert_eq!(workshop_recipes.len(), 2);

        // House has no recipes in defaults
        let house_recipes: Vec<_> = catalog.for_building(BuildingType::House).collect();
        assert_eq!(house_recipes.len(), 0);
    }

    #[test]
    fn test_recipe_catalog_get_nonexistent() {
        let catalog = RecipeCatalog::with_defaults();
        assert!(catalog.get("nonexistent_recipe").is_none());
    }

    #[test]
    fn test_recipe_catalog_add() {
        let mut catalog = RecipeCatalog::new();
        assert!(catalog.all().is_empty());

        catalog.add(Recipe {
            id: "custom".into(),
            name: "Custom Recipe".into(),
            building_type: BuildingType::Granary,
            inputs: vec![(ResourceType::Food, 10)],
            outputs: vec![(ResourceType::Food, 5)], // Preservation - some loss
            work_required: 20,
            workers_needed: 1,
        });

        assert_eq!(catalog.all().len(), 1);
        assert!(catalog.get("custom").is_some());
    }

    #[test]
    fn test_recipe_toml_parsing() {
        let toml_content = r#"
[[recipes]]
id = "iron_smelting"
name = "Iron Smelting"
building_type = "Workshop"
work_required = 60
workers_needed = 2

[[recipes.inputs]]
resource = "Ore"
amount = 4

[[recipes.outputs]]
resource = "Iron"
amount = 2

[[recipes]]
id = "grain_farming"
name = "Grain Farming"
building_type = "Farm"
work_required = 120
workers_needed = 3
inputs = []

[[recipes.outputs]]
resource = "Food"
amount = 10
"#;

        let catalog = RecipeCatalog::parse_toml(toml_content).expect("Failed to parse TOML");

        // Check iron smelting recipe
        let iron = catalog.get("iron_smelting").expect("Should have iron_smelting");
        assert_eq!(iron.building_type, BuildingType::Workshop);
        assert_eq!(iron.work_required, 60);
        assert_eq!(iron.workers_needed, 2);
        assert_eq!(iron.inputs.len(), 1);
        assert_eq!(iron.inputs[0], (ResourceType::Ore, 4));
        assert_eq!(iron.outputs.len(), 1);
        assert_eq!(iron.outputs[0], (ResourceType::Iron, 2));

        // Check grain farming recipe
        let grain = catalog.get("grain_farming").expect("Should have grain_farming");
        assert_eq!(grain.building_type, BuildingType::Farm);
        assert!(grain.inputs.is_empty());
        assert_eq!(grain.outputs[0], (ResourceType::Food, 10));
    }

    #[test]
    fn test_recipe_toml_invalid_building_type() {
        let toml_content = r#"
[[recipes]]
id = "invalid"
name = "Invalid Recipe"
building_type = "InvalidBuilding"
work_required = 10
workers_needed = 1

[[recipes.outputs]]
resource = "Food"
amount = 1
"#;

        let result = RecipeCatalog::parse_toml(toml_content);
        assert!(result.is_err());
        match result.unwrap_err() {
            RecipeLoadError::InvalidBuildingType(t) => assert_eq!(t, "InvalidBuilding"),
            _ => panic!("Expected InvalidBuildingType error"),
        }
    }

    #[test]
    fn test_recipe_toml_invalid_resource_type() {
        let toml_content = r#"
[[recipes]]
id = "invalid"
name = "Invalid Recipe"
building_type = "Farm"
work_required = 10
workers_needed = 1

[[recipes.outputs]]
resource = "Mana"
amount = 1
"#;

        let result = RecipeCatalog::parse_toml(toml_content);
        assert!(result.is_err());
        match result.unwrap_err() {
            RecipeLoadError::InvalidResourceType(t) => assert_eq!(t, "Mana"),
            _ => panic!("Expected InvalidResourceType error"),
        }
    }

    #[test]
    fn test_recipe_toml_case_insensitive() {
        let toml_content = r#"
[[recipes]]
id = "case_test"
name = "Case Test"
building_type = "WORKSHOP"
work_required = 10
workers_needed = 1

[[recipes.inputs]]
resource = "wood"
amount = 1

[[recipes.outputs]]
resource = "IRON"
amount = 1
"#;

        let catalog = RecipeCatalog::parse_toml(toml_content).expect("Should parse");
        let recipe = catalog.get("case_test").expect("Should have recipe");
        assert_eq!(recipe.building_type, BuildingType::Workshop);
        assert_eq!(recipe.inputs[0].0, ResourceType::Wood);
        assert_eq!(recipe.outputs[0].0, ResourceType::Iron);
    }

    #[test]
    fn test_load_recipes_from_file() {
        use std::path::Path;

        let path = Path::new("data/recipes.toml");
        let catalog = RecipeCatalog::load_from_toml(path)
            .expect("Should load recipes from data/recipes.toml");

        // Verify expected recipes exist
        assert!(catalog.get("farm_food").is_some(), "Should have farm_food recipe");
        assert!(catalog.get("smelt_iron").is_some(), "Should have smelt_iron recipe");
        assert!(catalog.get("weave_cloth").is_some(), "Should have weave_cloth recipe");
        assert!(catalog.get("preserve_food").is_some(), "Should have preserve_food recipe");
        assert!(catalog.get("forge_tools").is_some(), "Should have forge_tools recipe");

        // Verify recipe counts by building type
        let farm_recipes: Vec<_> = catalog.for_building(BuildingType::Farm).collect();
        assert_eq!(farm_recipes.len(), 1);

        let workshop_recipes: Vec<_> = catalog.for_building(BuildingType::Workshop).collect();
        assert_eq!(workshop_recipes.len(), 3);

        let granary_recipes: Vec<_> = catalog.for_building(BuildingType::Granary).collect();
        assert_eq!(granary_recipes.len(), 1);

        // Verify iron smelting recipe details
        let smelt = catalog.get("smelt_iron").unwrap();
        assert_eq!(smelt.building_type, BuildingType::Workshop);
        assert_eq!(smelt.work_required, 50);
        assert_eq!(smelt.workers_needed, 1);
        assert_eq!(smelt.inputs, vec![(ResourceType::Ore, 3)]);
        assert_eq!(smelt.outputs, vec![(ResourceType::Iron, 1)]);
    }
}
