use crate::BaseUrlMode;
use crate::generators::Generator;
use crate::generators::common::extract_type_name;
use crate::generators::import_generator::ImportGenerator;
use crate::openapi::{OpenApiSchema, Operation, Parameter};
use anyhow::Result;
use std::collections::BTreeMap;

pub struct AngularGenerator {
    with_zod: bool,
    debug: bool,
    promises: bool,
    base_url_mode: BaseUrlMode,
}

impl Default for AngularGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl AngularGenerator {
    pub fn new() -> Self {
        Self {
            with_zod: false,
            debug: false,
            promises: false,
            base_url_mode: BaseUrlMode::Global,
        }
    }

    pub fn with_zod_validation(mut self, with_zod: bool) -> Self {
        self.with_zod = with_zod;
        self
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn with_promises(mut self, promises: bool) -> Self {
        self.promises = promises;
        self
    }

    pub fn with_base_url_mode(mut self, mode: BaseUrlMode) -> Self {
        self.base_url_mode = mode;
        self
    }
}

impl Generator for AngularGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut services = BTreeMap::new();

        if self.debug {
            println!("🔍 [DEBUG] Angular Generator: Starting endpoint processing");
        }

        // Store reference to the full schema for later lookup
        let full_schema = schema;

        // Group endpoints by tag
        if let Some(paths) = &schema.paths {
            for (path, path_item) in paths {
                if self.debug {
                    println!("🔍 [DEBUG] Processing path: {path}");
                }

                // Handle different HTTP methods
                if let Some(operation) = &path_item.get {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] GET {path} -> tag: {tag}");
                    }
                    self.add_operation_to_services(
                        &mut services,
                        "GET",
                        path,
                        operation,
                        full_schema,
                    )?;
                }
                if let Some(operation) = &path_item.post {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] POST {path} -> tag: {tag}");
                    }
                    self.add_operation_to_services(
                        &mut services,
                        "POST",
                        path,
                        operation,
                        full_schema,
                    )?;
                }
                if let Some(operation) = &path_item.put {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] PUT {path} -> tag: {tag}");
                    }
                    self.add_operation_to_services(
                        &mut services,
                        "PUT",
                        path,
                        operation,
                        full_schema,
                    )?;
                }
                if let Some(operation) = &path_item.delete {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] DELETE {path} -> tag: {tag}");
                    }
                    self.add_operation_to_services(
                        &mut services,
                        "DELETE",
                        path,
                        operation,
                        full_schema,
                    )?;
                }
                if let Some(operation) = &path_item.patch {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] PATCH {path} -> tag: {tag}");
                    }
                    self.add_operation_to_services(
                        &mut services,
                        "PATCH",
                        path,
                        operation,
                        full_schema,
                    )?;
                }
            }
        }

        // Collect tags for index generation
        let tags: Vec<String> = services.keys().cloned().collect();

        if self.debug {
            println!("🔍 [DEBUG] Found {} services: {:?}", services.len(), tags);
            for (tag, service_data) in &services {
                println!(
                    "🔍 [DEBUG] Service '{}' has {} methods",
                    tag,
                    service_data.methods.len()
                );
            }
        }

        // Generate all services
        let mut output = String::new();
        for (tag, service_data) in services {
            if self.debug {
                println!("🔍 [DEBUG] Generating service for tag: {tag}");
            }
            output.push_str(&self.generate_service_with_command(&tag, &service_data, command)?);
            output.push_str("\n\n");
        }

        // Generate index file
        output.push_str(&self.generate_index_with_command(&tags.iter().collect(), command)?);

        Ok(output)
    }
}

impl AngularGenerator {
    fn add_operation_to_services(
        &self,
        services: &mut BTreeMap<String, ServiceData>,
        method: &str,
        path: &str,
        operation: &Operation,
        full_schema: &OpenApiSchema,
    ) -> Result<()> {
        let tag = operation
            .tags
            .as_ref()
            .and_then(|tags| tags.first())
            .unwrap_or(&"Default".to_string())
            .clone();

        if !services.contains_key(&tag) {
            services.insert(
                tag.clone(),
                ServiceData {
                    imports: std::collections::HashSet::new(),
                    methods: Vec::new(),
                    response_types: std::collections::HashSet::new(),
                    request_types: std::collections::HashSet::new(),
                    query_param_types: std::collections::HashSet::new(),
                    has_void_methods: false,
                    has_header_params: false,
                    uses_z_methods: false,
                },
            );
        }

        let service_data = services.get_mut(&tag).unwrap();

        // Check if this is a void method
        let return_type = self.get_return_type(operation)?;
        if return_type == "void" {
            service_data.has_void_methods = true;
        }

        // Check if this method uses z.array() (for array return types)
        if self.with_zod && return_type.ends_with("[]") {
            service_data.uses_z_methods = true;
        }

        // Check if this operation has header parameters
        if let Some(parameters) = &operation.parameters {
            if parameters.iter().any(|p| p.location == "header") {
                service_data.has_header_params = true;
            }
        }

        // Generate method with schema for reference resolution
        let method_code = self.generate_method(method, path, operation, full_schema)?;
        service_data.methods.push(method_code);

        // Collect imports
        self.collect_imports(operation, service_data)?;

        Ok(())
    }

    fn generate_method(
        &self,
        http_method: &str,
        path: &str,
        operation: &Operation,
        full_schema: &OpenApiSchema,
    ) -> Result<String> {
        let method_name = self.get_method_name(operation);
        let parameters = self.get_method_parameters(operation)?;
        let return_type = self.get_return_type(operation)?;

        let mut method = String::new();

        // Generate TSDoc comment
        method.push_str(&self.generate_tsdoc_comment(
            http_method,
            path,
            operation,
            &return_type,
        )?);

        // For void types or when promises flag is set, return Promise instead of Observable
        if return_type == "void" || self.promises {
            method.push_str(&format!(
                "  {method_name}({parameters}): Promise<{return_type}> {{\n"
            ));
        } else {
            method.push_str(&format!(
                "  {method_name}({parameters}): Observable<{return_type}> {{\n"
            ));
        }

        // Generate URL building
        let path_template = self.format_path_template(path);

        if self.base_url_mode == BaseUrlMode::Global {
            method.push_str(&format!(
                "    const url = `${{this.baseUrl}}{path_template}`;\n"
            ));
        } else {
            method.push_str(&format!("    const url = `${{baseUrl}}{path_template}`;\n"));
        }

        // Check for multipart/form-data and generate FormData conversion
        let is_multipart = if let Some(request_body) = &operation.request_body {
            if let Some(content) = &request_body.content {
                content.contains_key("multipart/form-data")
            } else {
                false
            }
        } else {
            false
        };

        if is_multipart {
            // Generate explicit FormData conversion based on schema fields
            method.push_str("    const formData = new FormData();\n");

            if let Some(request_body) = &operation.request_body
                && let Some(content) = &request_body.content
                && let Some(media_type) = content.get("multipart/form-data")
                && let Some(schema) = &media_type.schema
            {
                self.generate_formdata_conversion(&mut method, schema, full_schema)?;
            }
        }

        // Check for query params
        let has_query_params = if let Some(parameters) = &operation.parameters {
            parameters.iter().any(|p| p.location == "query")
        } else {
            false
        };

        let options = if has_query_params {
            "{ headers, params: queryParams }"
        } else {
            "{ headers }"
        };

        // Generate HTTP call
        let request_body = self.get_request_body(operation)?;

        let http_call = match http_method {
            "GET" => {
                format!("this.http.get<{return_type}>(url, {options})")
            }
            "POST" => {
                if request_body.is_empty() {
                    format!("this.http.post<{return_type}>(url, null, {options})")
                } else {
                    format!("this.http.post<{return_type}>(url{request_body}, {options})")
                }
            }
            "PUT" => {
                if request_body.is_empty() {
                    format!("this.http.put<{return_type}>(url, null, {options})")
                } else {
                    format!("this.http.put<{return_type}>(url{request_body}, {options})")
                }
            }
            "DELETE" => {
                format!("this.http.delete<{return_type}>(url, {options})")
            }
            "PATCH" => {
                if request_body.is_empty() {
                    format!("this.http.patch<{return_type}>(url, null, {options})")
                } else {
                    format!("this.http.patch<{return_type}>(url{request_body}, {options})")
                }
            }
            _ => {
                if has_query_params {
                    format!(
                        "this.http.request<{return_type}>('{http_method}', {{ url, headers, params: queryParams }})"
                    )
                } else {
                    format!("this.http.request<{return_type}>('{http_method}', {{ url, headers }})")
                }
            }
        };

        // Add Zod validation for response if enabled (but not for requests)
        if self.with_zod {
            // Skip validation for void types - they shouldn't be validated
            if return_type == "void" {
                method.push_str(&format!("    return lastValueFrom({http_call});\n"));
            } else {
                let response_schema_name = if return_type == "unknown[]" {
                    // For unknown arrays, we can't validate the schema, so just skip validation
                    "z.array(z.unknown())".to_string()
                } else if return_type.ends_with("[]") {
                    // For typed arrays, create array schema
                    let base_type = &return_type[..return_type.len() - 2];
                    format!("z.array({base_type}Schema)")
                } else {
                    // For single types, use the standard schema
                    format!("{return_type}Schema")
                };

                if self.promises {
                    // When promises flag is set, wrap the observable with lastValueFrom
                    method.push_str(&format!("    return lastValueFrom({http_call}\n"));
                    method.push_str("      .pipe(\n");

                    if return_type == "unknown[]" {
                        // For unknown arrays, just cast to the expected type without validation
                        method.push_str(&format!(
                            "        map(response => response as {return_type})\n"
                        ));
                    } else {
                        method.push_str(&format!(
                            "        map(response => {response_schema_name}.parse(response))\n"
                        ));
                    }

                    method.push_str("      ));\n");
                } else {
                    // Original observable behavior
                    method.push_str(&format!("    return {http_call}\n"));
                    method.push_str("      .pipe(\n");

                    if return_type == "unknown[]" {
                        // For unknown arrays, just cast to the expected type without validation
                        method.push_str(&format!(
                            "        map(response => response as {return_type})\n"
                        ));
                    } else {
                        method.push_str(&format!(
                            "        map(response => {response_schema_name}.parse(response))\n"
                        ));
                    }

                    method.push_str("      );\n");
                }
            }
        } else {
            // For void types or when promises flag is set, convert Observable to Promise
            if return_type == "void" || self.promises {
                method.push_str(&format!("    return lastValueFrom({http_call});\n"));
            } else {
                method.push_str(&format!("    return {http_call};\n"));
            }
        }

        method.push_str("  }\n");

        Ok(method)
    }

    fn get_method_name(&self, operation: &Operation) -> String {
        if let Some(summary) = &operation.summary {
            let camel_case = summary
                .split_whitespace()
                .enumerate()
                .map(|(i, word)| {
                    if i == 0 {
                        word.to_lowercase()
                    } else {
                        word.chars()
                            .next()
                            .map(|c| c.to_uppercase().collect::<String>() + &word[1..])
                            .unwrap_or_default()
                    }
                })
                .collect::<String>();
            camel_case
        } else {
            "unknownMethod".to_string()
        }
    }

    fn get_method_parameters(&self, operation: &Operation) -> Result<String> {
        let mut params = Vec::new();

        // Add mandatory baseUrl parameter as the first parameter for consistency
        if self.base_url_mode != BaseUrlMode::Global {
            params.push("baseUrl: string".to_string());
        }

        // Path parameters
        if let Some(parameters) = &operation.parameters {
            for param in parameters {
                if param.location == "path" {
                    let param_type = self.get_parameter_type(param);
                    params.push(format!(
                        "{}: {}",
                        self.to_camel_case(&param.name),
                        param_type
                    ));
                }
            }
        }

        // Query parameters (with named types)
        if let Some(parameters) = &operation.parameters {
            let query_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "query")
                .collect();

            if !query_params.is_empty() {
                let has_mandatory = query_params.iter().any(|p| p.required.unwrap_or(false));
                let optional_marker = if has_mandatory { "" } else { "?" };

                if let Some(type_name) = self.get_query_param_type_name(operation) {
                    params.push(format!("queryParams{optional_marker}: {type_name}"));
                } else {
                    // Fallback to inline type if no good name can be generated
                    let mut query_type = "{ ".to_string();
                    for (i, param) in query_params.iter().enumerate() {
                        let param_type = self.get_parameter_type(param);
                        let optional = if param.required.unwrap_or(false) {
                            ""
                        } else {
                            "?"
                        };
                        query_type.push_str(&format!("{}{}: {}", param.name, optional, param_type));
                        if i < query_params.len() - 1 {
                            query_type.push_str(", ");
                        }
                    }
                    query_type.push_str(" }");
                    params.push(format!("queryParams{optional_marker}: {query_type}"));
                }
            }
        }

        // Request body
        if let Some(request_body) = &operation.request_body
            && let Some(content) = &request_body.content
        {
            if let Some(media_type) = content.get("application/json")
                && let Some(schema) = &media_type.schema
            {
                let type_name = self.get_schema_type_name(schema);
                params.push(format!("dto: {type_name}"));
            } else if let Some(media_type) = content.get("multipart/form-data")
                && let Some(schema) = &media_type.schema
            {
                let type_name = self.get_schema_type_name(schema);
                params.push(format!("data: {type_name}"));
            }
        }

        // Merge header parameters and HttpHeaders into single parameter
        if let Some(parameters) = &operation.parameters {
            let header_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "header")
                .collect();

            if !header_params.is_empty() {
                if let Some(type_name) = self.get_header_param_type_name(operation) {
                    params.push(format!("headers?: {type_name} & Record<string, string>"));
                } else {
                    // Fallback to inline type if no good name can be generated
                    let mut header_type = "{ ".to_string();
                    for (i, param) in header_params.iter().enumerate() {
                        let param_type = self.get_parameter_type(param);
                        let optional = if param.required.unwrap_or(false) {
                            ""
                        } else {
                            "?"
                        };
                        header_type
                            .push_str(&format!("\"{}\"{}:{}", param.name, optional, param_type));
                        if i < header_params.len() - 1 {
                            header_type.push_str(", ");
                        }
                    }
                    header_type.push_str(" } & Record<string, string>");
                    params.push(format!("headers?: {header_type}"));
                }
            } else {
                // No header parameters, just add HttpHeaders
                params.push("headers?: HttpHeaders".to_string());
            }
        } else {
            // No parameters at all, just add HttpHeaders
            params.push("headers?: HttpHeaders".to_string());
        }

        Ok(params.join(", "))
    }

    fn get_return_type(&self, operation: &Operation) -> Result<String> {
        if let Some(responses) = &operation.responses
            && let Some(success_response) = responses.get("200").or_else(|| responses.get("201"))
            && let Some(content) = &success_response.content
            && let Some(media_type) = content.get("application/json")
            && let Some(schema) = &media_type.schema
        {
            return Ok(self.get_schema_type_name(schema));
        }
        Ok("void".to_string())
    }

    fn get_request_body(&self, operation: &Operation) -> Result<String> {
        if let Some(request_body) = &operation.request_body {
            if let Some(content) = &request_body.content {
                if content.contains_key("multipart/form-data") {
                    Ok(", formData".to_string())
                } else {
                    Ok(", dto".to_string())
                }
            } else {
                Ok(", dto".to_string())
            }
        } else {
            Ok("".to_string())
        }
    }

    fn collect_imports(&self, operation: &Operation, service_data: &mut ServiceData) -> Result<()> {
        // Collect response types (always import both type and schema when using Zod)
        if let Some(responses) = &operation.responses
            && let Some(success_response) = responses.get("200").or_else(|| responses.get("201"))
            && let Some(content) = &success_response.content
            && let Some(media_type) = content.get("application/json")
            && let Some(schema) = &media_type.schema
            && let Some(type_name) = extract_type_name(schema)
        {
            service_data.imports.insert(type_name.clone());
            if self.with_zod {
                service_data.response_types.insert(type_name);
            }
        }

        // Collect request body types (always import the TypeScript type, but don't import schema when using Zod)
        if let Some(request_body) = &operation.request_body
            && let Some(content) = &request_body.content
        {
            if let Some(media_type) = content.get("application/json")
                && let Some(schema) = &media_type.schema
                && let Some(type_name) = extract_type_name(schema)
            {
                service_data.imports.insert(type_name.clone());
                if self.with_zod {
                    service_data.request_types.insert(type_name);
                }
            } else if let Some(media_type) = content.get("multipart/form-data")
                && let Some(schema) = &media_type.schema
                && let Some(type_name) = extract_type_name(schema)
            {
                service_data.imports.insert(type_name.clone());
                if self.with_zod {
                    service_data.request_types.insert(type_name);
                }
            }
        }

        // Collect parameter types (for query, path, and header parameters)
        if let Some(parameters) = &operation.parameters {
            let query_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "query")
                .collect();

            // If there are query parameters and we can generate a good type name, add it to imports
            if !query_params.is_empty() {
                if let Some(type_name) = self.get_query_param_type_name(operation) {
                    service_data.imports.insert(type_name.clone());
                    service_data.query_param_types.insert(type_name);
                }
            }

            // Collect header parameter types
            let header_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "header")
                .collect();

            if !header_params.is_empty() {
                if let Some(type_name) = self.get_header_param_type_name(operation) {
                    service_data.imports.insert(type_name.clone());
                    service_data.query_param_types.insert(type_name);
                }
            }

            // Don't collect parameter schema types - they're only referenced in JSDoc comments
            // and would result in unused imports
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn generate_service(&self, tag: &str, service_data: &ServiceData) -> Result<String> {
        self.generate_service_with_command(tag, service_data, "dtolator")
    }

    fn generate_service_with_command(
        &self,
        tag: &str,
        service_data: &ServiceData,
        command: &str,
    ) -> Result<String> {
        let class_name = format!("{tag}Api");
        let file_name = self.to_kebab_case(&format!("{tag}-api"));

        let mut service = String::new();

        // Add file marker BEFORE the content for proper splitting
        service.push_str("// FILE: ");
        service.push_str(&file_name);
        service.push_str(".ts\n");

        service.push_str(&format!("// Generated by {command}\n"));
        service.push_str("// Do not modify manually\n\n");

        // Build imports using ImportGenerator
        let mut import_gen = ImportGenerator::new();

        // Angular imports
        import_gen.add_imports(
            "@angular/common/http",
            vec!["HttpClient", "HttpHeaders"],
            false,
        );
        import_gen.add_imports("@angular/core", vec!["Injectable", "inject"], false);

        // RxJS imports
        if !self.promises {
            import_gen.add_import("rxjs", "Observable", true);
        }
        if service_data.has_void_methods || self.promises {
            import_gen.add_import("rxjs", "lastValueFrom", false);
        }
        if self.with_zod {
            import_gen.add_import("rxjs/operators", "map", false);
            if service_data.uses_z_methods {
                import_gen.add_import("zod", "z", false);
            }
        }

        // Local imports

        if !service_data.imports.is_empty() {
            let imports: Vec<String> = service_data.imports.iter().cloned().collect();

            if self.with_zod {
                // Import types from dto.ts
                for import in &imports {
                    import_gen.add_import("./dto", import, true);
                }

                // Import schemas separately (these are runtime values)
                let response_types_to_import: Vec<_> = imports
                    .iter()
                    .filter(|name| service_data.response_types.contains(*name))
                    .collect();

                for import in &response_types_to_import {
                    import_gen.add_import("./dto", &format!("{import}Schema"), false);
                }
            } else {
                // Import only types
                for import in &imports {
                    import_gen.add_import("./dto", import, true);
                }
            }
        }

        service.push_str(&import_gen.generate());
        service.push('\n');

        // Service class
        service.push_str("@Injectable({ providedIn: 'root' })\n");
        service.push_str(&format!("export class {class_name} {{\n"));
        service.push_str("  private http = inject(HttpClient);\n");

        if self.base_url_mode == BaseUrlMode::Global {
            service.push_str("  private baseUrl: string;\n\n");
            service.push_str("  constructor() {\n");
            service.push_str("    this.baseUrl = (globalThis as any).API_URL || (typeof window !== 'undefined' && (window as any).API_URL);\n");
            service
                .push_str("    if (!this.baseUrl) throw new Error('API_URL is not configured');\n");
            service.push_str("  }\n\n");
        } else {
            service.push_str("\n");
        }

        // Methods
        for method in &service_data.methods {
            service.push_str(method);
            service.push('\n');
        }

        service.push_str("}\n");

        Ok(service)
    }

    fn generate_index_with_command(&self, tags: &Vec<&String>, command: &str) -> Result<String> {
        let mut index = String::new();

        // Add file marker BEFORE the content for proper splitting
        index.push_str("// FILE: index.ts\n");

        index.push_str(&format!("// Generated by {command}\n"));
        index.push_str("// Do not modify manually\n\n");

        // Sort exports alphabetically
        let mut exports = Vec::new();
        exports.push("\"./dto\"".to_string());

        for tag in tags {
            let file_name = self.to_kebab_case(&format!("{tag}-api"));
            exports.push(format!("\"./{file_name}\""));
        }
        exports.sort();

        for export in exports {
            index.push_str(&format!("export * from {export};\n"));
        }

        Ok(index)
    }

    // Helper methods
    fn extract_path_params(&self, path: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = path.chars();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut param = String::new();
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                    param.push(ch);
                }
                if !param.is_empty() {
                    params.push(param);
                }
            }
        }

        params
    }

    fn get_parameter_type(&self, parameter: &Parameter) -> String {
        if let Some(schema) = &parameter.schema {
            // Use the same logic as get_schema_type_name to handle references properly
            self.get_schema_type_name(schema)
        } else {
            "unknown".to_string()
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn get_schema_type_name(&self, schema: &crate::openapi::Schema) -> String {
        match schema {
            crate::openapi::Schema::Reference { reference } => reference
                .strip_prefix("#/components/schemas/")
                .unwrap_or(reference)
                .to_string(),
            crate::openapi::Schema::Object {
                schema_type, items, ..
            } => match schema_type.as_deref() {
                Some("string") => "string".to_string(),
                Some("number") | Some("integer") => "number".to_string(),
                Some("boolean") => "boolean".to_string(),
                Some("array") => {
                    if let Some(items_schema) = items {
                        let item_type = self.get_schema_type_name(items_schema);
                        format!("{item_type}[]")
                    } else {
                        "unknown[]".to_string()
                    }
                }
                Some("object") => "Record<string, unknown>".to_string(),
                _ => "unknown".to_string(),
            },
        }
    }

    fn to_camel_case(&self, input: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = false;

        for ch in input.chars() {
            if ch == '_' || ch == '-' {
                capitalize_next = true;
            } else if capitalize_next {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn to_kebab_case(&self, input: &str) -> String {
        input.replace("_", "-").to_lowercase()
    }

    fn to_pascal_case(&self, input: &str) -> String {
        input
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect()
    }

    fn generate_formdata_conversion(
        &self,
        method: &mut String,
        schema: &crate::openapi::Schema,
        full_schema: &OpenApiSchema,
    ) -> Result<()> {
        // Resolve the schema if it's a reference
        let resolved_schema = self.resolve_schema_ref(schema, full_schema);

        match resolved_schema {
            crate::openapi::Schema::Object {
                properties,
                required,
                ..
            } => {
                if let Some(props) = properties {
                    let required_fields: std::collections::HashSet<String> = required
                        .as_ref()
                        .map(|r| r.iter().cloned().collect())
                        .unwrap_or_default();

                    for (prop_name, prop_schema) in props {
                        let is_optional = !required_fields.contains(prop_name);

                        // Determine if this property is a file/blob or needs string conversion
                        let is_binary = matches!(
                            prop_schema,
                            crate::openapi::Schema::Object {
                                format: Some(f),
                                ..
                            } if f == "binary"
                        );

                        let is_array = matches!(
                            prop_schema,
                            crate::openapi::Schema::Object {
                                schema_type: Some(t),
                                ..
                            } if t == "array"
                        );

                        let is_string = matches!(
                            prop_schema,
                            crate::openapi::Schema::Object {
                                schema_type: Some(t),
                                ..
                            } if t == "string"
                        );

                        let is_object = matches!(
                            prop_schema,
                            crate::openapi::Schema::Object {
                                schema_type: Some(t),
                                ..
                            } if t == "object"
                        );

                        if is_optional {
                            method.push_str(&format!("    if (data.{}) {{\n", prop_name));
                            if is_array {
                                method.push_str(&format!(
                                    "      data.{}.forEach(item => formData.append('{}', item));\n",
                                    prop_name, prop_name
                                ));
                            } else if is_binary || is_string {
                                // Files/blobs and strings can be appended directly
                                method.push_str(&format!(
                                    "      formData.append('{}', data.{});\n",
                                    prop_name, prop_name
                                ));
                            } else if is_object {
                                // Objects need to be stringified as JSON
                                method.push_str(&format!(
                                    "      formData.append('{}', JSON.stringify(data.{}));\n",
                                    prop_name, prop_name
                                ));
                            } else {
                                // Numbers, booleans, etc. need to be converted to string
                                method.push_str(&format!(
                                    "      formData.append('{}', String(data.{}));\n",
                                    prop_name, prop_name
                                ));
                            }
                            method.push_str("    }\n");
                        } else {
                            if is_array {
                                method.push_str(&format!(
                                    "    data.{}.forEach(item => formData.append('{}', item));\n",
                                    prop_name, prop_name
                                ));
                            } else if is_binary || is_string {
                                // Files/blobs and strings can be appended directly
                                method.push_str(&format!(
                                    "    formData.append('{}', data.{});\n",
                                    prop_name, prop_name
                                ));
                            } else if is_object {
                                // Objects need to be stringified as JSON
                                method.push_str(&format!(
                                    "    formData.append('{}', JSON.stringify(data.{}));\n",
                                    prop_name, prop_name
                                ));
                            } else {
                                // Numbers, booleans, etc. need to be converted to string
                                method.push_str(&format!(
                                    "    formData.append('{}', String(data.{}));\n",
                                    prop_name, prop_name
                                ));
                            }
                        }
                    }
                }
            }
            crate::openapi::Schema::Reference { .. } => {
                // For references, we can't resolve at generation time, so fall back to generic approach
                method.push_str("    Object.entries(data).forEach(([key, value]) => {\n");
                method.push_str("      if (Array.isArray(value)) {\n");
                method.push_str("        value.forEach(item => formData.append(key, item));\n");
                method.push_str("      } else if (value !== undefined && value !== null) {\n");
                method.push_str("        if (typeof value === 'object' && !(value instanceof Blob) && !(value instanceof File)) {\n");
                method.push_str("          formData.append(key, JSON.stringify(value));\n");
                method.push_str("        } else if (typeof value === 'string' || value instanceof Blob || value instanceof File) {\n");
                method.push_str("          formData.append(key, value);\n");
                method.push_str("        } else {\n");
                method.push_str("          formData.append(key, String(value));\n");
                method.push_str("        }\n");
                method.push_str("      }\n");
                method.push_str("    });\n");
            }
        }

        Ok(())
    }

    fn get_query_param_type_name(&self, operation: &Operation) -> Option<String> {
        if let Some(summary) = &operation.summary {
            // Convert "Get Sales Analytics" -> "GetSalesAnalyticsQueryParams"
            let clean_summary = summary
                .replace("Get ", "")
                .replace("Create ", "")
                .replace("Update ", "")
                .replace("Delete ", "")
                .replace("Retrieve ", "")
                .replace("Fetch ", "");
            let pascal_name = self.to_pascal_case(&clean_summary);
            Some(format!("{pascal_name}QueryParams"))
        } else {
            None
        }
    }

    fn get_header_param_type_name(&self, operation: &Operation) -> Option<String> {
        if let Some(summary) = &operation.summary {
            let clean_summary = summary
                .replace("Get ", "")
                .replace("Create ", "")
                .replace("Update ", "")
                .replace("Delete ", "")
                .replace("Retrieve ", "")
                .replace("Fetch ", "");
            let pascal_name = self.to_pascal_case(&clean_summary);
            Some(format!("{pascal_name}Headers"))
        } else {
            None
        }
    }

    /// Generate the fill-url utility function
    fn format_path_template(&self, path: &str) -> String {
        let mut template = path.to_string();
        if !template.starts_with('/') {
            template.insert(0, '/');
        }

        let path_params = self.extract_path_params(path);
        for param in path_params {
            let camel_param = self.to_camel_case(&param);
            let replacement = format!("${{encodeURIComponent({camel_param})}}");
            template = template.replace(&format!("{{{param}}}"), &replacement);
        }

        template
    }

    pub fn generate_query_param_types(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut types = String::new();
        let mut generated_types = std::collections::HashSet::new();

        if let Some(paths) = &schema.paths {
            for (_path, path_item) in paths {
                let operations = [
                    &path_item.get,
                    &path_item.post,
                    &path_item.put,
                    &path_item.delete,
                    &path_item.patch,
                ];

                for operation in operations.into_iter().flatten() {
                    if let Some(parameters) = &operation.parameters {
                        let query_params: Vec<&Parameter> = parameters
                            .iter()
                            .filter(|p| p.location == "query")
                            .collect();

                        if !query_params.is_empty() {
                            if let Some(type_name) = self.get_query_param_type_name(operation) {
                                if !generated_types.contains(&type_name) {
                                    generated_types.insert(type_name.clone());

                                    // Add JSDoc comment for the interface
                                    if let Some(summary) = &operation.summary {
                                        types.push_str(&format!(
                                            "/**\n * Query parameters for {summary}\n */\n"
                                        ));
                                    }

                                    let mandatory_params: Vec<&Parameter> = query_params
                                        .iter()
                                        .filter(|p| p.required.unwrap_or(false))
                                        .cloned()
                                        .collect();

                                    let optional_params: Vec<&Parameter> = query_params
                                        .iter()
                                        .filter(|p| !p.required.unwrap_or(false))
                                        .cloned()
                                        .collect();

                                    types.push_str(&format!("export type {type_name} = "));

                                    if !mandatory_params.is_empty() {
                                        types.push_str("{\n");
                                        for param in &mandatory_params {
                                            let param_type = self.get_parameter_type(param);
                                            types.push_str(&format!(
                                                "  {}: {};\n",
                                                param.name, param_type
                                            ));
                                        }
                                        types.push('}');
                                    }

                                    if !optional_params.is_empty() {
                                        if !mandatory_params.is_empty() {
                                            types.push_str(" & ");
                                        }
                                        types.push_str("Partial<{\n");
                                        for param in &optional_params {
                                            let param_type = self.get_parameter_type(param);
                                            types.push_str(&format!(
                                                "  {}: {};\n",
                                                param.name, param_type
                                            ));
                                        }
                                        types.push_str("}>");
                                    }

                                    types.push_str(";\n\n");
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(types)
    }

    pub fn generate_header_param_types(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut types = String::new();
        let mut generated_types = std::collections::HashSet::new();

        if let Some(paths) = &schema.paths {
            for (_path, path_item) in paths {
                let operations = [
                    &path_item.get,
                    &path_item.post,
                    &path_item.put,
                    &path_item.delete,
                    &path_item.patch,
                ];

                for operation in operations.into_iter().flatten() {
                    if let Some(parameters) = &operation.parameters {
                        let header_params: Vec<&Parameter> = parameters
                            .iter()
                            .filter(|p| p.location == "header")
                            .collect();

                        if !header_params.is_empty() {
                            if let Some(type_name) = self.get_header_param_type_name(operation) {
                                if !generated_types.contains(&type_name) {
                                    generated_types.insert(type_name.clone());

                                    if let Some(summary) = &operation.summary {
                                        types.push_str(&format!(
                                            "/**\n * Header parameters for {summary}\n */\n"
                                        ));
                                    }

                                    types.push_str(&format!("export interface {type_name} {{\n"));
                                    for param in &header_params {
                                        let param_type = self.get_parameter_type(param);
                                        let optional = if param.required.unwrap_or(false) {
                                            ""
                                        } else {
                                            "?"
                                        };

                                        types.push_str(&format!(
                                            "  \"{}\"{}: {};\n",
                                            param.name, optional, param_type
                                        ));
                                    }
                                    types.push_str("}\n\n");
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(types)
    }

    fn generate_tsdoc_comment(
        &self,
        http_method: &str,
        path: &str,
        operation: &Operation,
        return_type: &str,
    ) -> Result<String> {
        let mut comment = String::new();
        comment.push_str("  /**\n");

        // Add summary as the main description
        if let Some(summary) = &operation.summary {
            comment.push_str(&format!("   * {summary}\n"));
        } else {
            comment.push_str(&format!("   * {} {}\n", http_method.to_uppercase(), path));
        }

        // Add detailed description if available
        if let Some(description) = &operation.description {
            comment.push_str("   *\n");
            comment.push_str(&format!("   * {description}\n"));
        }

        comment.push_str("   *\n");

        // Document mandatory baseUrl parameter first for consistency
        if self.base_url_mode != BaseUrlMode::Global {
            comment.push_str("   * @param baseUrl - Base URL for the request\n");
        }

        // Document path parameters
        if let Some(parameters) = &operation.parameters {
            let path_params: Vec<&Parameter> =
                parameters.iter().filter(|p| p.location == "path").collect();

            for param in path_params {
                let param_name = self.to_camel_case(&param.name);
                let param_type = self.get_parameter_type(param);
                comment.push_str(&format!(
                    "   * @param {param_name} - Path parameter of type {param_type}\n"
                ));
            }
        }

        // Document query parameters
        if let Some(parameters) = &operation.parameters {
            let query_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "query")
                .collect();

            if !query_params.is_empty() {
                comment.push_str("   * @param queryParams - Query parameters object\n");
                for param in query_params {
                    let required = if param.required.unwrap_or(false) {
                        "required"
                    } else {
                        "optional"
                    };
                    let param_type = self.get_parameter_type(param);
                    comment.push_str(&format!(
                        "   * @param queryParams.{} - {} parameter of type {}\n",
                        param.name, required, param_type
                    ));
                }
            }
        }

        // Document request body
        if let Some(request_body) = &operation.request_body
            && let Some(content) = &request_body.content
            && let Some(media_type) = content.get("application/json")
            && let Some(schema) = &media_type.schema
        {
            let type_name = self.get_schema_type_name(schema);
            let description = request_body
                .description
                .as_ref()
                .map(|d| format!(" - {d}"))
                .unwrap_or_default();
            comment.push_str(&format!(
                "   * @param dto - Request body of type {type_name}{description}\n"
            ));
        }

        // Document headers parameter
        if let Some(parameters) = &operation.parameters {
            let header_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "header")
                .collect();

            if !header_params.is_empty() {
                comment.push_str(
                    "   * @param headers - Custom header values or HttpHeaders instance\n",
                );
                for param in header_params {
                    let required = if param.required.unwrap_or(false) {
                        "required"
                    } else {
                        "optional"
                    };
                    let param_type = self.get_parameter_type(param);
                    comment.push_str(&format!(
                        "   * @param headers.{} - {} header of type {}\n",
                        param.name, required, param_type
                    ));
                }
            } else {
                comment.push_str("   * @param headers - Optional HTTP headers\n");
            }
        } else {
            comment.push_str("   * @param headers - Optional HTTP headers\n");
        }

        // Document return type
        let return_wrapper = if return_type == "void" || self.promises {
            "Promise"
        } else {
            "Observable"
        };

        if let Some(responses) = &operation.responses {
            if let Some(success_response) = responses.get("200").or_else(|| responses.get("201")) {
                let response_desc = &success_response.description;
                comment.push_str(&format!(
                    "   * @returns {return_wrapper}<{return_type}> - {response_desc}\n"
                ));
            } else {
                comment.push_str(&format!("   * @returns {return_wrapper}<{return_type}>\n"));
            }
        } else {
            comment.push_str(&format!("   * @returns {return_wrapper}<{return_type}>\n"));
        }

        comment.push_str("   */\n");

        Ok(comment)
    }

    /// Resolve a schema reference by looking it up in components.schemas
    fn resolve_schema_ref<'a>(
        &self,
        schema: &'a crate::openapi::Schema,
        full_schema: &'a OpenApiSchema,
    ) -> &'a crate::openapi::Schema {
        match schema {
            crate::openapi::Schema::Reference { reference } => {
                // Extract the schema name from the reference and look it up in components.schemas
                if let Some(schema_name) = reference.strip_prefix("#/components/schemas/")
                    && let Some(components) = &full_schema.components
                    && let Some(schemas) = &components.schemas
                    && let Some(resolved) = schemas.get(schema_name)
                {
                    return resolved;
                }
                // If we can't resolve, return the original
                schema
            }
            _ => schema,
        }
    }
}

#[derive(Debug)]
struct ServiceData {
    imports: std::collections::HashSet<String>,
    methods: Vec<String>,
    response_types: std::collections::HashSet<String>,
    request_types: std::collections::HashSet<String>,
    query_param_types: std::collections::HashSet<String>,
    has_void_methods: bool,
    has_header_params: bool,
    uses_z_methods: bool,
}
