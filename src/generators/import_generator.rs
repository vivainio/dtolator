use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportCategory {
    Builtin,  // node:* modules
    External, // npm packages
    Internal, // relative paths
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StatementType {
    Import,
    Export,
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub is_type_only: bool,
    pub source: String,
    pub imports: Vec<String>,
    pub category: ImportCategory,
    pub statement_type: StatementType,
}

impl ImportStatement {
    fn categorize_source(source: &str) -> ImportCategory {
        if source.starts_with("node:") {
            ImportCategory::Builtin
        } else if source.starts_with('.') {
            ImportCategory::Internal
        } else {
            ImportCategory::External
        }
    }

    fn format_single_line(&self) -> String {
        let (keyword, from_keyword) = match self.statement_type {
            StatementType::Import => {
                let import_keyword = if self.is_type_only {
                    "import type "
                } else {
                    "import "
                };
                (import_keyword, "from")
            }
            StatementType::Export => {
                let export_keyword = if self.is_type_only {
                    "export type "
                } else {
                    "export "
                };
                (export_keyword, "from")
            }
        };

        let imports_str = self.imports.join(", ");
        format!(
            "{keyword}{{ {imports_str} }} {from_keyword} \"{}\";",
            self.source
        )
    }

    fn format_multi_line(&self) -> String {
        let (keyword, from_keyword) = match self.statement_type {
            StatementType::Import => {
                let import_keyword = if self.is_type_only {
                    "import type "
                } else {
                    "import "
                };
                (import_keyword, "from")
            }
            StatementType::Export => {
                let export_keyword = if self.is_type_only {
                    "export type "
                } else {
                    "export "
                };
                (export_keyword, "from")
            }
        };

        let mut result = format!("{keyword}{{\n");
        for import in &self.imports {
            result.push_str(&format!("  {import},\n"));
        }
        result.push_str(&format!("}} {from_keyword} \"{}\";\n", self.source));
        result
    }

    fn should_split(&self) -> bool {
        // Calculate line length for both import and export statements
        let prefix = match self.statement_type {
            StatementType::Import => {
                if self.is_type_only {
                    "import type { "
                } else {
                    "import { "
                }
            }
            StatementType::Export => {
                if self.is_type_only {
                    "export type { "
                } else {
                    "export { "
                }
            }
        };
        let line_length = prefix.len()
            + self.imports.join(", ").len()
            + " } from \"".len()
            + self.source.len()
            + "\";".len();

        line_length > 80
    }

    pub fn format(&self) -> String {
        if self.should_split() {
            self.format_multi_line()
        } else {
            format!("{}\n", self.format_single_line())
        }
    }
}

pub struct ImportGenerator {
    // Map of (source, is_type_only, statement_type) -> list of import/export names
    statements: BTreeMap<(String, bool, StatementType), Vec<String>>,
}

impl ImportGenerator {
    pub fn new() -> Self {
        Self {
            statements: BTreeMap::new(),
        }
    }

    pub fn add_import(&mut self, source: &str, name: &str, is_type: bool) {
        self.add_statement(source, name, is_type, StatementType::Import);
    }

    pub fn add_imports(&mut self, source: &str, names: Vec<&str>, all_types: bool) {
        for name in names {
            self.add_import(source, name, all_types);
        }
    }

    pub fn add_export(&mut self, source: &str, name: &str, is_type: bool) {
        self.add_statement(source, name, is_type, StatementType::Export);
    }

    pub fn add_exports(&mut self, source: &str, names: Vec<&str>, all_types: bool) {
        for name in names {
            self.add_export(source, name, all_types);
        }
    }

    fn add_statement(
        &mut self,
        source: &str,
        name: &str,
        is_type: bool,
        statement_type: StatementType,
    ) {
        let key = (source.to_string(), is_type, statement_type);
        let items = self.statements.entry(key).or_default();
        // Only add if not already present (deduplicate)
        if !items.contains(&name.to_string()) {
            items.push(name.to_string());
        }
    }

    fn build_statements(&self) -> Vec<ImportStatement> {
        let mut statements = Vec::new();

        for ((source, is_type_only, statement_type), items) in &self.statements {
            let mut sorted_items = items.clone();
            sorted_items.sort();

            statements.push(ImportStatement {
                is_type_only: *is_type_only,
                source: source.clone(),
                imports: sorted_items,
                category: ImportStatement::categorize_source(source),
                statement_type: *statement_type,
            });
        }

        // Sort by statement type (imports first, then exports), category, then source
        statements.sort_by(|a, b| {
            (a.statement_type as i32)
                .cmp(&(b.statement_type as i32))
                .then_with(|| a.category.cmp(&b.category))
                .then_with(|| a.source.cmp(&b.source))
        });

        statements
    }

    pub fn generate(&self) -> String {
        let statements = self.build_statements();
        if statements.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        let mut last_category: Option<ImportCategory> = None;
        let mut last_statement_type: Option<StatementType> = None;

        for statement in statements {
            // Add blank line between statement types or categories
            if let (Some(last_type), Some(last_cat)) = (last_statement_type, &last_category)
                && (last_type != statement.statement_type || last_cat != &statement.category)
            {
                output.push('\n');
            }

            output.push_str(&statement.format());
            last_category = Some(statement.category.clone());
            last_statement_type = Some(statement.statement_type);
        }

        output
    }
}

impl Default for ImportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorization() {
        assert_eq!(
            ImportStatement::categorize_source("node:fs"),
            ImportCategory::Builtin
        );
        assert_eq!(
            ImportStatement::categorize_source("@angular/core"),
            ImportCategory::External
        );
        assert_eq!(
            ImportStatement::categorize_source("./schema"),
            ImportCategory::Internal
        );
    }

    #[test]
    fn test_single_line_import_format() {
        let stmt = ImportStatement {
            is_type_only: false,
            source: "zod".to_string(),
            imports: vec!["z".to_string()],
            category: ImportCategory::External,
            statement_type: StatementType::Import,
        };
        assert_eq!(stmt.format(), "import { z } from \"zod\";\n");
    }

    #[test]
    fn test_single_line_export_format() {
        let stmt = ImportStatement {
            is_type_only: false,
            source: "./schema".to_string(),
            imports: vec!["User".to_string(), "Product".to_string()],
            category: ImportCategory::Internal,
            statement_type: StatementType::Export,
        };
        let formatted = stmt.format();
        // Items should be sorted alphabetically
        assert!(formatted.contains("Product"));
        assert!(formatted.contains("User"));
        assert!(formatted.contains("export {"));
        assert!(formatted.contains("} from \"./schema\";"));
    }

    #[test]
    fn test_multi_line_format() {
        let stmt = ImportStatement {
            is_type_only: true,
            source: "@angular/common/http".to_string(),
            imports: vec![
                "HttpClient".to_string(),
                "HttpHeaders".to_string(),
                "HttpContext".to_string(),
                "HttpParams".to_string(),
            ],
            category: ImportCategory::External,
            statement_type: StatementType::Import,
        };
        let formatted = stmt.format();
        assert!(formatted.contains("import type {\n"));
        assert!(formatted.contains("  HttpClient,\n"));
    }

    #[test]
    fn test_export_type_multi_line() {
        let mut sorted_imports = vec![
            "UserProfileSchema".to_string(),
            "ProductSchema".to_string(),
            "OrderSchema".to_string(),
            "CustomerSchema".to_string(),
            "InventorySchema".to_string(),
            "PaymentMethodSchema".to_string(),
        ];
        sorted_imports.sort();

        let stmt = ImportStatement {
            is_type_only: true,
            source: "./schema".to_string(),
            imports: sorted_imports,
            category: ImportCategory::Internal,
            statement_type: StatementType::Export,
        };
        let formatted = stmt.format();
        assert!(formatted.contains("export type {\n"));
        assert!(formatted.contains("  CustomerSchema,\n"));
    }

    #[test]
    fn test_import_generator() {
        let mut generator = ImportGenerator::new();
        generator.add_import("@angular/core", "Injectable", false);
        generator.add_imports(
            "@angular/common/http",
            vec!["HttpClient", "HttpHeaders"],
            true,
        );
        generator.add_import("./dto", "User", true);

        let output = generator.generate();

        // External imports should come before internal
        let angular_pos = output.find("@angular").unwrap();
        let dto_pos = output.find("./dto").unwrap();
        assert!(angular_pos < dto_pos);

        // Should have blank line between categories
        assert!(output.contains("\n\n"));
    }

    #[test]
    fn test_import_and_export_generator() {
        let mut generator = ImportGenerator::new();
        // Imports first
        generator.add_import("./schema", "User", true);
        generator.add_import("./schema", "Product", true);
        // Exports
        generator.add_export("./schema", "UserSchema", false);
        generator.add_export("./schema", "ProductSchema", false);

        let output = generator.generate();

        // Imports should come before exports
        let import_pos = output.find("import").unwrap();
        let export_pos = output.find("export").unwrap();
        assert!(import_pos < export_pos);

        // Should have blank line between import and export sections
        assert!(output.contains("\n\n"));
    }
}
