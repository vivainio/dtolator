use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImportCategory {
    Builtin,  // node:* modules
    External, // npm packages
    Internal, // relative paths
}

#[derive(Debug, Clone)]
pub struct ImportStatement {
    pub is_type_only: bool,
    pub source: String,
    pub imports: Vec<String>,
    pub category: ImportCategory,
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
        let import_type = if self.is_type_only {
            "import type "
        } else {
            "import "
        };

        let imports_str = self.imports.join(", ");
        format!("{import_type}{{ {imports_str} }} from \"{}\";", self.source)
    }

    fn format_multi_line(&self) -> String {
        let import_type = if self.is_type_only {
            "import type "
        } else {
            "import "
        };

        let mut result = format!("{import_type}{{\n");
        for import in &self.imports {
            result.push_str(&format!("  {import},\n"));
        }
        result.push_str(&format!("}} from \"{}\";\n", self.source));
        result
    }

    fn should_split(&self) -> bool {
        // Calculate line length: "import type { X, Y, Z } from "source";"
        let import_prefix = if self.is_type_only {
            "import type { "
        } else {
            "import { "
        };
        let line_length = import_prefix.len()
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
    // Map of (source, is_type_only) -> list of import names
    imports: BTreeMap<(String, bool), Vec<String>>,
}

impl ImportGenerator {
    pub fn new() -> Self {
        Self {
            imports: BTreeMap::new(),
        }
    }

    pub fn add_import(&mut self, source: &str, name: &str, is_type: bool) {
        let key = (source.to_string(), is_type);
        let imports = self.imports.entry(key).or_default();
        // Only add if not already present (deduplicate)
        if !imports.contains(&name.to_string()) {
            imports.push(name.to_string());
        }
    }

    pub fn add_imports(&mut self, source: &str, names: Vec<&str>, all_types: bool) {
        for name in names {
            self.add_import(source, name, all_types);
        }
    }

    fn build_import_statements(&self) -> Vec<ImportStatement> {
        let mut statements = Vec::new();

        for ((source, is_type_only), imports) in &self.imports {
            let mut sorted_imports = imports.clone();
            sorted_imports.sort();

            statements.push(ImportStatement {
                is_type_only: *is_type_only,
                source: source.clone(),
                imports: sorted_imports,
                category: ImportStatement::categorize_source(source),
            });
        }

        // Sort by category first, then by source
        statements.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| a.source.cmp(&b.source))
        });

        statements
    }

    pub fn generate(&self) -> String {
        let statements = self.build_import_statements();
        if statements.is_empty() {
            return String::new();
        }

        let mut output = String::new();
        let mut last_category: Option<ImportCategory> = None;

        for statement in statements {
            // Add blank line between categories
            if let Some(ref last_cat) = last_category {
                if *last_cat != statement.category {
                    output.push('\n');
                }
            }

            output.push_str(&statement.format());
            last_category = Some(statement.category.clone());
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
    fn test_single_line_format() {
        let stmt = ImportStatement {
            is_type_only: false,
            source: "zod".to_string(),
            imports: vec!["z".to_string()],
            category: ImportCategory::External,
        };
        assert_eq!(stmt.format(), "import { z } from \"zod\";\n");
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
        };
        let formatted = stmt.format();
        assert!(formatted.contains("import type {\n"));
        assert!(formatted.contains("  HttpClient,\n"));
    }

    #[test]
    fn test_import_generator() {
        let mut gen = ImportGenerator::new();
        gen.add_import("@angular/core", "Injectable", false);
        gen.add_imports(
            "@angular/common/http",
            vec!["HttpClient", "HttpHeaders"],
            true,
        );
        gen.add_import("./dto", "User", true);

        let output = gen.generate();

        // External imports should come before internal
        let angular_pos = output.find("@angular").unwrap();
        let dto_pos = output.find("./dto").unwrap();
        assert!(angular_pos < dto_pos);

        // Should have blank line between categories
        assert!(output.contains("\n\n"));
    }
}
