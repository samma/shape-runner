use serde::{Deserialize, Serialize};

use crate::types::{FieldDef, TypeDef};

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureDesignInput {
    pub repo_summary: String,
    pub constraints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureDesignOutput {
    pub name: String,
    pub rationale: String, // treat Markdown as plain String
    pub components: Vec<Component>,
    pub risks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    pub id: String,
    pub responsibility: String,
    pub api: String,
}

// TypeDef for FeatureDesignOutput (for validation of LLM JSON)
pub fn feature_design_output_typedef() -> TypeDef {
    TypeDef::Object(vec![
        FieldDef {
            name: "name",
            ty: TypeDef::Text,
        },
        FieldDef {
            name: "rationale",
            ty: TypeDef::Markdown,
        },
        FieldDef {
            name: "components",
            ty: TypeDef::List(Box::new(TypeDef::Object(vec![
                FieldDef {
                    name: "id",
                    ty: TypeDef::Text,
                },
                FieldDef {
                    name: "responsibility",
                    ty: TypeDef::Text,
                },
                FieldDef {
                    name: "api",
                    ty: TypeDef::Markdown,
                },
            ]))),
        },
        FieldDef {
            name: "risks",
            ty: TypeDef::List(Box::new(TypeDef::Text)),
        },
    ])
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormationInput {
    pub formation_description: String,
    pub unit_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FormationOutput {
    pub coordinates: Vec<Coordinate>,
}

// TypeDef for FormationOutput (for validation of LLM JSON)
pub fn formation_output_typedef() -> TypeDef {
    TypeDef::Object(vec![
        FieldDef {
            name: "coordinates",
            ty: TypeDef::List(Box::new(TypeDef::Object(vec![
                FieldDef {
                    name: "x",
                    ty: TypeDef::Number,
                },
                FieldDef {
                    name: "y",
                    ty: TypeDef::Number,
                },
            ]))),
        },
    ])
}
