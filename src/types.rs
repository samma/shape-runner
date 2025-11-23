use serde_json::Value;

/// Simple type system for shapes.
#[derive(Debug, Clone)]
pub enum TypeDef {
    Text,
    Markdown,
    Number,
    Bool,
    List(Box<TypeDef>),
    Object(Vec<FieldDef>),
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: &'static str,
    pub ty: TypeDef,
}

/// Single validation error, with a JSON path.
#[derive(Debug, Clone)]
pub enum ValidationError {
    MissingField { path: String },
    TypeMismatch { path: String, expected: &'static str, found: &'static str },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingField { path } => {
                write!(f, "Missing required field at path {path}")
            }
            ValidationError::TypeMismatch { path, expected, found } => {
                write!(f, "Type mismatch at {path}: expected {expected}, found {found}")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a serde_json::Value against a TypeDef.
///
/// Returns Ok(()) if everything matches, or Err(vec![]) with one or more errors.
pub fn validate(ty: &TypeDef, value: &Value) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    validate_inner(ty, value, "$", &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_inner(ty: &TypeDef, value: &Value, path: &str, errors: &mut Vec<ValidationError>) {
    use TypeDef::*;

    match ty {
        Text | Markdown => {
            if !value.is_string() {
                errors.push(ValidationError::TypeMismatch {
                    path: path.to_string(),
                    expected: "string",
                    found: value_type_name(value),
                });
            }
        }
        Number => {
            if !value.is_number() {
                errors.push(ValidationError::TypeMismatch {
                    path: path.to_string(),
                    expected: "number",
                    found: value_type_name(value),
                });
            }
        }
        Bool => {
            if !value.is_boolean() {
                errors.push(ValidationError::TypeMismatch {
                    path: path.to_string(),
                    expected: "boolean",
                    found: value_type_name(value),
                });
            }
        }
        List(inner) => {
            if let Value::Array(items) = value {
                for (idx, item) in items.iter().enumerate() {
                    let child_path = format!("{path}[{idx}]");
                    validate_inner(inner, item, &child_path, errors);
                }
            } else {
                errors.push(ValidationError::TypeMismatch {
                    path: path.to_string(),
                    expected: "array",
                    found: value_type_name(value),
                });
            }
        }
        Object(fields) => {
            let Some(obj) = value.as_object() else {
                errors.push(ValidationError::TypeMismatch {
                    path: path.to_string(),
                    expected: "object",
                    found: value_type_name(value),
                });
                return;
            };

            for field in fields {
                let field_value = obj.get(field.name);
                let field_path = format!("{path}.{}", field.name);

                match field_value {
                    None => {
                        errors.push(ValidationError::MissingField { path: field_path });
                    }
                    Some(v) => {
                        validate_inner(&field.ty, v, &field_path, errors);
                    }
                }
            }

            // Extra fields are ignored (lenient). Can tighten later.
        }
    }
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
