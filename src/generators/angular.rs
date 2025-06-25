use anyhow::Result;
use crate::openapi::{OpenApiSchema, Operation, Parameter};
use crate::generators::Generator;
use std::collections::BTreeMap;

pub struct AngularGenerator {
    base_url: String,
    with_zod: bool,
}

impl AngularGenerator {
    pub fn new() -> Self {
        Self {
            base_url: "environment.apiUrl".to_string(),
            with_zod: false,
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
}

impl Generator for AngularGenerator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut services = BTreeMap::new();
        
        // Group endpoints by tag
        if let Some(paths) = &schema.paths {
            for (path, path_item) in paths {
                // Handle different HTTP methods
                if let Some(operation) = &path_item.get {
                    self.add_operation_to_services(&mut services, "GET", path, operation)?;
                }
                if let Some(operation) = &path_item.post {
                    self.add_operation_to_services(&mut services, "POST", path, operation)?;
                }
                if let Some(operation) = &path_item.put {
                    self.add_operation_to_services(&mut services, "PUT", path, operation)?;
                }
                if let Some(operation) = &path_item.delete {
                    self.add_operation_to_services(&mut services, "DELETE", path, operation)?;
                }
                if let Some(operation) = &path_item.patch {
                    self.add_operation_to_services(&mut services, "PATCH", path, operation)?;
                }
            }
        }
        
        // Collect tags for index generation
        let tags: Vec<String> = services.keys().cloned().collect();
        
        // Generate all services
        let mut output = String::new();
        for (tag, service_data) in services {
            output.push_str(&self.generate_service(&tag, &service_data)?);
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
        let tag = operation.tags.as_ref()
            .and_then(|tags| tags.first())
            .unwrap_or(&"Default".to_string())
            .clone();
        
        if !services.contains_key(&tag) {
            services.insert(tag.clone(), ServiceData {
                imports: std::collections::HashSet::new(),
                methods: Vec::new(),
            });
        }
        
        let service_data = services.get_mut(&tag).unwrap();
        
        // Generate method
        let method_code = self.generate_method(method, path, operation)?;
        service_data.methods.push(method_code);
        
        // Collect imports
        self.collect_imports(operation, service_data)?;
        
        Ok(())
    }
    
    fn generate_method(&self, http_method: &str, path: &str, operation: &Operation) -> Result<String> {
        let method_name = self.get_method_name(operation);
        let parameters = self.get_method_parameters(operation)?;
        let return_type = self.get_return_type(operation)?;
        
        let mut method = String::new();
        method.push_str(&format!("  {}({}): Observable<{}> {{\n", method_name, parameters, return_type));
        
        // Add Zod validation for request body if enabled
        if self.with_zod {
            if let Some(request_body) = &operation.request_body {
                if let Some(content) = &request_body.content {
                    if let Some(media_type) = content.get("application/json") {
                        if let Some(schema) = &media_type.schema {
                            let schema_name = format!("{}Schema", self.get_schema_type_name(schema));
                            method.push_str(&format!("    // Validate request body with Zod\n"));
                            method.push_str(&format!("    const validatedDto = {}.parse(dto);\n", schema_name));
                            method.push_str("\n");
                        }
                    }
                }
            }
        }
        
        // Generate URL building
        let url_params = self.get_url_params(path, operation)?;
        let query_params = self.get_query_params(operation)?;
        
        method.push_str(&format!("    const url = subsToUrl(\"{}\", {}, {});\n", path, url_params, query_params));
        
        // Generate HTTP call
        let request_body = if self.with_zod && operation.request_body.is_some() {
            ", validatedDto".to_string()
        } else {
            self.get_request_body(operation)?
        };
        
        let http_call = match http_method {
            "GET" => format!("this.http.get<{}>(url)", return_type),
            "POST" => format!("this.http.post<{}>(url{})", return_type, request_body),
            "PUT" => format!("this.http.put<{}>(url{})", return_type, request_body),
            "DELETE" => format!("this.http.delete<{}>(url)", return_type),
            "PATCH" => format!("this.http.patch<{}>(url{})", return_type, request_body),
            _ => format!("this.http.request<{}>('{}', {{ url }})", return_type, http_method),
        };
        
        // Add Zod validation for response if enabled
        if self.with_zod {
            let response_schema_name = format!("{}Schema", return_type);
            method.push_str(&format!("    return {}\n", http_call));
            method.push_str("      .pipe(\n");
            method.push_str(&format!("        map(response => {}.parse(response))\n", response_schema_name));
            method.push_str("      );\n");
        } else {
            method.push_str(&format!("    return {};\n", http_call));
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
                    params.push(format!("{}: {}", self.to_camel_case(&param.name), param_type));
                }
            }
        }
        
        // Query parameters
        if let Some(parameters) = &operation.parameters {
            let query_params: Vec<&Parameter> = parameters.iter()
                .filter(|p| p.location == "query")
                .collect();
            
            if !query_params.is_empty() {
                let mut query_type = "{ ".to_string();
                for (i, param) in query_params.iter().enumerate() {
                    let param_type = self.get_parameter_type(param);
                    let optional = if param.required.unwrap_or(false) { "" } else { "?" };
                    query_type.push_str(&format!("{}{}: {}", param.name, optional, param_type));
                    if i < query_params.len() - 1 {
                        query_type.push_str(", ");
                    }
                }
                query_type.push_str(" }");
                params.push(format!("queryParams?: {}", query_type));
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
            let query_params: Vec<&Parameter> = parameters.iter()
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
        // Collect response types
        if let Some(responses) = &operation.responses {
            if let Some(success_response) = responses.get("200").or_else(|| responses.get("201")) {
                if let Some(content) = &success_response.content {
                    if let Some(media_type) = content.get("application/json") {
                        if let Some(schema) = &media_type.schema {
                            if let Some(type_name) = self.extract_type_name(schema) {
                                service_data.imports.insert(type_name);
                            }
                        }
                    }
                }
            }
        }
        
        // Collect request body types
        if let Some(request_body) = &operation.request_body {
            if let Some(content) = &request_body.content {
                if let Some(media_type) = content.get("application/json") {
                    if let Some(schema) = &media_type.schema {
                        if let Some(type_name) = self.extract_type_name(schema) {
                            service_data.imports.insert(type_name);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn extract_type_name(&self, schema: &crate::openapi::Schema) -> Option<String> {
        match schema {
            crate::openapi::Schema::Reference { reference } => {
                Some(reference.strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string())
            }
            _ => None,
        }
    }
    
    fn generate_service(&self, tag: &str, service_data: &ServiceData) -> Result<String> {
        let class_name = format!("{}Api", tag);
        let file_name = self.to_kebab_case(&format!("{}-api", tag));
        
        let mut service = String::new();
        
        // Header comment
        service.push_str("// Generated Angular service from OpenAPI schema\n");
        service.push_str("// Do not modify this file manually\n\n");
        
        // Imports
        service.push_str("import { HttpClient } from \"@angular/common/http\";\n");
        service.push_str("import { Injectable } from \"@angular/core\";\n");
        service.push_str("import { Observable } from \"rxjs\";\n");
        
        if self.with_zod {
            service.push_str("import { map } from \"rxjs/operators\";\n");
        }
        
        service.push_str("import { subsToUrl } from \"./subs-to-url.func\";\n");
        
        if !service_data.imports.is_empty() {
            let mut imports: Vec<String> = service_data.imports.iter().cloned().collect();
            imports.sort();
            
            if self.with_zod {
                // Import both types and schemas when using Zod in multi-line format
                service.push_str("import {\n");
                for import in &imports {
                    service.push_str(&format!("  {}Schema,\n", import));
                    service.push_str(&format!("  type {},\n", import));
                }
                service.push_str("} from \"./dto\";\n");
            } else {
                // Import only types in multi-line format
                service.push_str("import {\n");
                for import in &imports {
                    service.push_str(&format!("  {},\n", import));
                }
                service.push_str("} from \"./dto\";\n");
            }
        }
        
        service.push_str("\n");
        
        // Service class
        service.push_str("@Injectable({ providedIn: \"root\" })\n");
        service.push_str(&format!("export class {} {{\n", class_name));
        service.push_str("  constructor(private http: HttpClient) {}\n\n");
        
        // Methods
        for method in &service_data.methods {
            service.push_str(method);
            service.push_str("\n");
        }
        
        service.push_str("}\n");
        
        // Add file marker for output splitting
        service.push_str(&format!("\n// FILE: {}.ts\n", file_name));
        
        Ok(service)
    }
    
    fn generate_index(&self, tags: &Vec<&String>) -> Result<String> {
        let mut index = String::new();
        
        index.push_str("// Generated index file for Angular services\n");
        index.push_str("// Do not modify this file manually\n\n");
        
        index.push_str("export * from \"./dto\";\n");
        index.push_str("export * from \"./subs-to-url.func\";\n");
        
        for tag in tags {
            let file_name = self.to_kebab_case(&format!("{}-api", tag));
            index.push_str(&format!("export * from \"./{}\";\n", file_name));
        }
        
        index.push_str("\n// FILE: index.ts\n");
        
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
            match schema.get_type() {
                Some("string") => "string".to_string(),
                Some("number") | Some("integer") => "number".to_string(),
                Some("boolean") => "boolean".to_string(),
                Some("array") => "string[]".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            "unknown".to_string()
        }
    }
    
    fn get_schema_type_name(&self, schema: &crate::openapi::Schema) -> String {
        match schema {
            crate::openapi::Schema::Reference { reference } => {
                reference.strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string()
            }
            crate::openapi::Schema::Object { schema_type, .. } => {
                match schema_type.as_deref() {
                    Some("string") => "string".to_string(),
                    Some("number") | Some("integer") => "number".to_string(),
                    Some("boolean") => "boolean".to_string(),
                    Some("array") => "unknown[]".to_string(),
                    Some("object") => "Record<string, unknown>".to_string(),
                    _ => "unknown".to_string(),
                }
            }
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
}

#[derive(Debug)]
struct ServiceData {
    imports: std::collections::HashSet<String>,
    methods: Vec<String>,
} 