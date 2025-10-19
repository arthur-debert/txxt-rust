//! # Processing Stages and Format Registries
//!
//! This module defines the core data structures and registries for managing
//! the different stages of the TXXT parsing pipeline and the available output formats.
//!
//! ## Key Components:
//!
//! * **`Stage`**: Represents a specific point in the parsing pipeline, like "token-scanner" or "ast-full".
//! * **`Format`**: Represents an output format, such as "json" or "treeviz".
//! * **`StageRegistry`**: A singleton that holds all registered `Stage`s.
//! * **`FormatRegistry`**: A singleton that holds all registered `Format`s.
//! * **`ConversionFactory`**: A singleton that links `Stage`s to their supported `Format`s.
//!
//! This setup allows for a flexible and extensible CLI, where new stages and formats
//! can be added without modifying the core logic of the binary.

use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

// --- Data Structures ---

/// Represents a single stage in the parsing pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Stage {
    pub name: &'static str,
    pub description: &'static str,
    pub data_structure: &'static str,
}

/// Represents an output format.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Format {
    pub name: &'static str,
    pub description: &'static str,
}

// --- Registries ---

pub struct StageRegistry {
    stages: HashMap<&'static str, Stage>,
}

impl StageRegistry {
    fn new() -> Self {
        Self {
            stages: HashMap::new(),
        }
    }

    pub fn register(&mut self, stage: Stage) {
        self.stages.insert(stage.name, stage);
    }

    pub fn get(&self, name: &str) -> Option<&Stage> {
        self.stages.get(name)
    }

    pub fn list(&self) -> Vec<&Stage> {
        self.stages.values().collect()
    }
}

pub struct FormatRegistry {
    formats: HashMap<&'static str, Format>,
}

impl FormatRegistry {
    fn new() -> Self {
        Self {
            formats: HashMap::new(),
        }
    }

    pub fn register(&mut self, format: Format) {
        self.formats.insert(format.name, format);
    }

    pub fn get(&self, name: &str) -> Option<&Format> {
        self.formats.get(name)
    }

    pub fn list(&self) -> Vec<&Format> {
        self.formats.values().collect()
    }
}

// --- Conversion Factory ---

pub struct ConversionFactory {
    conversions: HashMap<&'static str, HashSet<&'static str>>,
}

impl ConversionFactory {
    fn new() -> Self {
        Self {
            conversions: HashMap::new(),
        }
    }

    pub fn register(&mut self, data_structure: &'static str, formats: Vec<&'static str>) {
        let format_set = formats.into_iter().collect();
        self.conversions.insert(data_structure, format_set);
    }

    pub fn is_supported(
        &self,
        stage_name: &str,
        format_name: &str,
        stage_registry: &StageRegistry,
    ) -> bool {
        if let Some(stage) = stage_registry.get(stage_name) {
            if let Some(supported_formats) = self.conversions.get(stage.data_structure) {
                return supported_formats.contains(format_name);
            }
        }
        false
    }
}

// --- Global Static Instances ---

pub static STAGE_REGISTRY: Lazy<Mutex<StageRegistry>> =
    Lazy::new(|| Mutex::new(StageRegistry::new()));
pub static FORMAT_REGISTRY: Lazy<Mutex<FormatRegistry>> =
    Lazy::new(|| Mutex::new(FormatRegistry::new()));
pub static CONVERSION_FACTORY: Lazy<Mutex<ConversionFactory>> =
    Lazy::new(|| Mutex::new(ConversionFactory::new()));

// --- Initialization ---

/// Initializes and populates all the registries.
pub fn initialize_registries() {
    let mut stage_registry = STAGE_REGISTRY.lock().unwrap();
    let mut format_registry = FORMAT_REGISTRY.lock().unwrap();
    let mut conversion_factory = CONVERSION_FACTORY.lock().unwrap();

    // Register Stages
    stage_registry.register(Stage {
        name: "scanner-tokens",
        description: "Raw scanner tokens",
        data_structure: "token-scanner",
    });
    stage_registry.register(Stage {
        name: "semantic-tokens",
        description: "Semantically analyzed tokens",
        data_structure: "token-semantic",
    });
    stage_registry.register(Stage {
        name: "ast-block",
        description: "Block-level Abstract Syntax Tree",
        data_structure: "ast-block",
    });
    stage_registry.register(Stage {
        name: "ast-inlines",
        description: "AST with parsed inlines",
        data_structure: "ast-inlines",
    });
    stage_registry.register(Stage {
        name: "ast-document",
        description: "Document-level AST",
        data_structure: "ast-document",
    });
    stage_registry.register(Stage {
        name: "ast-full",
        description: "Full AST with annotations",
        data_structure: "ast-full",
    });

    // Register Formats
    format_registry.register(Format {
        name: "json",
        description: "JSON output",
    });
    format_registry.register(Format {
        name: "treeviz",
        description: "Tree visualization",
    });

    // Register Conversions
    conversion_factory.register("token-scanner", vec!["json"]);
    conversion_factory.register("token-semantic", vec!["json"]);
    conversion_factory.register("ast-block", vec!["json", "treeviz"]);
    conversion_factory.register("ast-inlines", vec!["json", "treeviz"]);
    conversion_factory.register("ast-document", vec!["json", "treeviz"]);
    conversion_factory.register("ast-full", vec!["json", "treeviz"]);
}
