use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Operation, Parameter};
use anyhow::Result;
use std::collections::BTreeMap;

pub struct AngularGenerator {
    base_url: String,
    with_zod: bool,
    debug: bool,
    promises: bool,
}

impl AngularGenerator {
    pub fn new() -> Self {
        Self {
            base_url: "environment.apiUrl".to_string(),
            with_zod: false,
            debug: false,
            promises: false,
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
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
}

impl Generator for AngularGenerator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String> {
        self.generate_with_command(schema, "dtolator")
    }

    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut services = BTreeMap::new();

        if self.debug {
            println!("🔍 [DEBUG] Angular Generator: Starting endpoint processing");
        }

        // Group endpoints by tag
        if let Some(paths) = &schema.paths {
            for (path, path_item) in paths {
                if self.debug {
                    println!("🔍 [DEBUG] Processing path: {}", path);
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
                        println!("🔍 [DEBUG] GET {} -> tag: {}", path, tag);
                    }
                    self.add_operation_to_services(&mut services, "GET", path, operation)?;
                }
                if let Some(operation) = &path_item.post {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] POST {} -> tag: {}", path, tag);
                    }
                    self.add_operation_to_services(&mut services, "POST", path, operation)?;
                }
                if let Some(operation) = &path_item.put {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] PUT {} -> tag: {}", path, tag);
                    }
                    self.add_operation_to_services(&mut services, "PUT", path, operation)?;
                }
                if let Some(operation) = &path_item.delete {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] DELETE {} -> tag: {}", path, tag);
                    }
                    self.add_operation_to_services(&mut services, "DELETE", path, operation)?;
                }
                if let Some(operation) = &path_item.patch {
                    if self.debug {
                        let default_tag = "Default".to_string();
                        let tag = operation
                            .tags
                            .as_ref()
                            .and_then(|tags| tags.first())
                            .unwrap_or(&default_tag);
                        println!("🔍 [DEBUG] PATCH {} -> tag: {}", path, tag);
                    }
                    self.add_operation_to_services(&mut services, "PATCH", path, operation)?;
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
                println!("🔍 [DEBUG] Generating service for tag: {}", tag);
            }
            output.push_str(&self.generate_service_with_command(&tag, &service_data, command)?);
            output.push_str("\n\n");
        }

        // Generate index file
        output.push_str(&self.generate_index(&tags.iter().collect())?);

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
                },
            );
        }

        let service_data = services.get_mut(&tag).unwrap();

        // Check if this is a void method
        let return_type = self.get_return_type(operation)?;
        if return_type == "void" {
            service_data.has_void_methods = true;
        }

        // Generate method
        let method_code = self.generate_method(method, path, operation)?;
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
                "  {}({}): Promise<{}> {{\n",
                method_name, parameters, return_type
            ));
        } else {
            method.push_str(&format!(
                "  {}({}): Observable<{}> {{\n",
                method_name, parameters, return_type
            ));
        }

        // Generate URL building
        let url_params = self.get_url_params(path, operation)?;
        let query_params = self.get_query_params(operation)?;

        method.push_str(&format!(
            "    const url = fillUrl('{}', {}, {});\n",
            path, url_params, query_params
        ));

        // Generate HTTP call
        let request_body = self.get_request_body(operation)?;

        let http_call = match http_method {
            "GET" => format!("this.http.get<{}>(url)", return_type),
            "POST" => {
                if request_body.is_empty() {
                    format!("this.http.post<{}>(url, null)", return_type)
                } else {
                    format!("this.http.post<{}>(url{})", return_type, request_body)
                }
            }
            "PUT" => {
                if request_body.is_empty() {
                    format!("this.http.put<{}>(url, null)", return_type)
                } else {
                    format!("this.http.put<{}>(url{})", return_type, request_body)
                }
            }
            "DELETE" => format!("this.http.delete<{}>(url)", return_type),
            "PATCH" => {
                if request_body.is_empty() {
                    format!("this.http.patch<{}>(url, null)", return_type)
                } else {
                    format!("this.http.patch<{}>(url{})", return_type, request_body)
                }
            }
            _ => format!(
                "this.http.request<{}>('{}', {{ url }})",
                return_type, http_method
            ),
        };

        // Add Zod validation for response if enabled (but not for requests)
        if self.with_zod {
            // Skip validation for void types - they shouldn't be validated
            if return_type == "void" {
                method.push_str(&format!("    return lastValueFrom({});\n", http_call));
            } else {
                let response_schema_name = if return_type == "unknown[]" {
                    // For unknown arrays, we can't validate the schema, so just skip validation
                    format!("z.array(z.unknown())")
                } else if return_type.ends_with("[]") {
                    // For typed arrays, create array schema
                    let base_type = &return_type[..return_type.len() - 2];
                    format!("z.array({}Schema)", base_type)
                } else {
                    // For single types, use the standard schema
                    format!("{}Schema", return_type)
                };

                if self.promises {
                    // When promises flag is set, wrap the observable with lastValueFrom
                    method.push_str(&format!("    return lastValueFrom({}\n", http_call));
                    method.push_str("      .pipe(\n");

                    if return_type == "unknown[]" {
                        // For unknown arrays, just cast to the expected type without validation
                        method.push_str(&format!(
                            "        map(response => response as {})\n",
                            return_type
                        ));
                    } else {
                        method.push_str(&format!(
                            "        map(response => {}.parse(response))\n",
                            response_schema_name
                        ));
                    }

                    method.push_str("      ));\n");
                } else {
                    // Original observable behavior
                    method.push_str(&format!("    return {}\n", http_call));
                    method.push_str("      .pipe(\n");

                    if return_type == "unknown[]" {
                        // For unknown arrays, just cast to the expected type without validation
                        method.push_str(&format!(
                            "        map(response => response as {})\n",
                            return_type
                        ));
                    } else {
                        method.push_str(&format!(
                            "        map(response => {}.parse(response))\n",
                            response_schema_name
                        ));
                    }

                    method.push_str("      );\n");
                }
            }
        } else {
            // For void types or when promises flag is set, convert Observable to Promise
            if return_type == "void" || self.promises {
                method.push_str(&format!("    return lastValueFrom({});\n", http_call));
            } else {
                method.push_str(&format!("    return {};\n", http_call));
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
                if let Some(type_name) = self.get_query_param_type_name(operation) {
                    params.push(format!("queryParams?: {}", type_name));
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
                    params.push(format!("queryParams?: {}", query_type));
                }
            }
        }

        // Request body
        if let Some(request_body) = &operation.request_body {
            if let Some(content) = &request_body.content {
                if let Some(media_type) = content.get("application/json") {
                    if let Some(schema) = &media_type.schema {
                        let type_name = self.get_schema_type_name(schema);
                        params.push(format!("dto: {}", type_name));
                    }
                }
            }
        }

        Ok(params.join(", "))
    }

    fn get_return_type(&self, operation: &Operation) -> Result<String> {
        if let Some(responses) = &operation.responses {
            if let Some(success_response) = responses.get("200").or_else(|| responses.get("201")) {
                if let Some(content) = &success_response.content {
                    if let Some(media_type) = content.get("application/json") {
                        if let Some(schema) = &media_type.schema {
                            return Ok(self.get_schema_type_name(schema));
                        }
                    }
                }
            }
        }
        Ok("void".to_string())
    }

    fn get_url_params(&self, path: &str, _operation: &Operation) -> Result<String> {
        let path_params = self.extract_path_params(path);
        if path_params.is_empty() {
            return Ok("{}".to_string());
        }

        let mut params = Vec::new();
        for param in path_params {
            params.push(format!("{}: {}", param, self.to_camel_case(&param)));
        }

        Ok(format!("{{ {} }}", params.join(", ")))
    }

    fn get_query_params(&self, operation: &Operation) -> Result<String> {
        if let Some(parameters) = &operation.parameters {
            let query_params: Vec<&Parameter> = parameters
                .iter()
                .filter(|p| p.location == "query")
                .collect();

            if !query_params.is_empty() {
                return Ok("queryParams || {}".to_string());
            }
        }
        Ok("{}".to_string())
    }

    fn get_request_body(&self, operation: &Operation) -> Result<String> {
        if let Some(_) = &operation.request_body {
            Ok(", dto".to_string())
        } else {
            Ok("".to_string())
        }
    }

    fn collect_imports(&self, operation: &Operation, service_data: &mut ServiceData) -> Result<()> {
        // Collect response types (always import both type and schema when using Zod)
        if let Some(responses) = &operation.responses {
            if let Some(success_response) = responses.get("200").or_else(|| responses.get("201")) {
                if let Some(content) = &success_response.content {
                    if let Some(media_type) = content.get("application/json") {
                        if let Some(schema) = &media_type.schema {
                            if let Some(type_name) = self.extract_type_name(schema) {
                                service_data.imports.insert(type_name.clone());
                                if self.with_zod {
                                    service_data.response_types.insert(type_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Collect request body types (always import the TypeScript type, but don't import schema when using Zod)
        if let Some(request_body) = &operation.request_body {
            if let Some(content) = &request_body.content {
                if let Some(media_type) = content.get("application/json") {
                    if let Some(schema) = &media_type.schema {
                        if let Some(type_name) = self.extract_type_name(schema) {
                            service_data.imports.insert(type_name.clone());
                            if self.with_zod {
                                service_data.request_types.insert(type_name);
                            }
                        }
                    }
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

            for parameter in parameters {
                if let Some(schema) = &parameter.schema {
                    if let Some(type_name) = self.extract_type_name(schema) {
                        service_data.imports.insert(type_name);
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_type_name(&self, schema: &crate::openapi::Schema) -> Option<String> {
        match schema {
            crate::openapi::Schema::Reference { reference } => Some(
                reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string(),
            ),
            _ => None,
        }
    }

    fn generate_service(&self, tag: &str, service_data: &ServiceData) -> Result<String> {
        self.generate_service_with_command(tag, service_data, "dtolator")
    }

    fn generate_service_with_command(
        &self,
        tag: &str,
        service_data: &ServiceData,
        command: &str,
    ) -> Result<String> {
        let class_name = format!("{}Api", tag);
        let file_name = self.to_kebab_case(&format!("{}-api", tag));

        let mut service = String::new();

        // Add file marker BEFORE the content for proper splitting
        service.push_str(&format!("// FILE: {}.ts\n", file_name));

        // Header comment
        service.push_str(&format!("// Generated by {}\n", command));
        service.push_str("// Do not modify manually\n\n");

        // Imports
        service.push_str("import { HttpClient } from '@angular/common/http';\n");
        service.push_str("import { Injectable } from '@angular/core';\n");

        // Import Observable only if not using promises
        if !self.promises {
            service.push_str("import { Observable } from 'rxjs';\n");
        }

        // Import lastValueFrom if there are void methods or promises are enabled
        if service_data.has_void_methods || self.promises {
            service.push_str("import { lastValueFrom } from 'rxjs';\n");
        }

        if self.with_zod {
            service.push_str("import { map } from 'rxjs/operators';\n");
            service.push_str("import { z } from 'zod';\n");
        }

        service.push_str("import { fillUrl } from './fill-url';\n");

        if !service_data.imports.is_empty() {
            let mut imports: Vec<String> = service_data.imports.iter().cloned().collect();
            imports.sort();

            if self.with_zod {
                // Import types and schemas from dto.ts (which has inferred types and re-exported schemas)
                service.push_str("import {\n");
                for import in &imports {
                    service.push_str(&format!("  {},\n", import));
                    // Only import schemas for response types, not request types
                    if service_data.response_types.contains(import) {
                        service.push_str(&format!("  {}Schema,\n", import));
                    }
                }
                service.push_str("} from './dto';\n");
            } else {
                // Import only types in multi-line format
                service.push_str("import {\n");
                for import in &imports {
                    service.push_str(&format!("  {},\n", import));
                }
                service.push_str("} from './dto';\n");
            }
        }

        service.push('\n');

        // Service class
        service.push_str("@Injectable({ providedIn: 'root' })\n");
        service.push_str(&format!("export class {} {{\n", class_name));
        service.push_str("  constructor(private http: HttpClient) {}\n\n");

        // Methods
        for method in &service_data.methods {
            service.push_str(method);
            service.push('\n');
        }

        service.push_str("}\n");

        Ok(service)
    }

    fn generate_index(&self, tags: &Vec<&String>) -> Result<String> {
        self.generate_index_with_command(tags, "dtolator")
    }

    fn generate_index_with_command(&self, tags: &Vec<&String>, command: &str) -> Result<String> {
        let mut index = String::new();

        // Add file marker BEFORE the content for proper splitting
        index.push_str("// FILE: index.ts\n");

        index.push_str(&format!("// Generated by {command}\n"));
        index.push_str("// Do not modify manually\n\n");

        index.push_str("export * from './dto';\n");
        index.push_str("export * from './fill-url';\n");

        for tag in tags {
            let file_name = self.to_kebab_case(&format!("{tag}-api"));
            index.push_str(&format!("export * from './{file_name}';\n"));
        }

        Ok(index)
    }

    // Helper methods
    fn extract_path_params(&self, path: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut chars = path.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                let mut param = String::new();
                while let Some(ch) = chars.next() {
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

    /// Generate the fill-url utility function
    pub fn generate_fill_url_func(&self, command_string: &str) -> String {
        let template = r#"// Generated by COMMAND_PLACEHOLDER
// Do not modify manually

type ParamValue = string | number | boolean | null | undefined;

export function fillUrl<T extends Record<string, any> = Record<string, ParamValue>>(
  url: string,
  params?: Record<string, ParamValue>,
  queryParams?: T,
): string {
  // Substitute path parameters - more efficient than regex for simple replacements
  if (params) {
    for (const [key, value] of Object.entries(params)) {
      if (value != null) {
        // Replace :param patterns, handling both end-of-string and followed by /
        const paramPattern = `:${key}`;
        let index = url.indexOf(paramPattern);
        while (index !== -1) {
          const nextChar = url[index + paramPattern.length];
          if (nextChar === undefined || nextChar === '/') {
            url = url.slice(0, index) + String(value) + url.slice(index + paramPattern.length);
            index = url.indexOf(paramPattern, index + String(value).length);
          } else {
            index = url.indexOf(paramPattern, index + 1);
          }
        }
      }
    }
  }

  // Build query string efficiently without intermediate arrays
  if (queryParams) {
    const queryParts: string[] = [];
    for (const [key, value] of Object.entries(queryParams)) {
      if (value != null) {
        queryParts.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
      }
    }
    
    if (queryParts.length > 0) {
      url += `?${queryParts.join('&')}`;
    }
  }

  // Get API base URL with improved error messaging
  const apiConfig = (globalThis as any).API_URL || (window as any)?.API_URL;
  if (!apiConfig) {
    throw new Error(
      'API_URL is not configured. Please set the global API_URL variable to your backend API base URL.\n' +
      'Examples:\n' +
      '  • Browser: (window as any).API_URL = "https://api.example.com";\n' +
      '  • Node.js: (globalThis as any).API_URL = "https://api.example.com";'
    );
  }

  // Ensure proper URL joining (handle trailing/leading slashes)
  const baseUrl = apiConfig.replace(/\/+$/, '');
  const path = url.startsWith('/') ? url : `/${url}`;
  
  return `${baseUrl}${path}`;
}
"#;
        template.replace("COMMAND_PLACEHOLDER", command_string)
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

                                    types.push_str(&format!("export interface {type_name} {{\n"));
                                    for param in &query_params {
                                        let param_type = self.get_parameter_type(param);
                                        let optional = if param.required.unwrap_or(false) {
                                            ""
                                        } else {
                                            "?"
                                        };

                                        types.push_str(&format!(
                                            "  {}{}: {};\n",
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
        if let Some(request_body) = &operation.request_body {
            if let Some(content) = &request_body.content {
                if let Some(media_type) = content.get("application/json") {
                    if let Some(schema) = &media_type.schema {
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
                }
            }
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
}

#[derive(Debug)]
struct ServiceData {
    imports: std::collections::HashSet<String>,
    methods: Vec<String>,
    response_types: std::collections::HashSet<String>,
    request_types: std::collections::HashSet<String>,
    query_param_types: std::collections::HashSet<String>,
    has_void_methods: bool,
}
