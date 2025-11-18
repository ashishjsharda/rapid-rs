// Template management for project generation
// Future: Add support for custom templates

pub struct Template {
    pub name: String,
    pub description: String,
}

pub fn available_templates() -> Vec<Template> {
    vec![
        Template {
            name: "rest-api".to_string(),
            description: "REST API with CRUD operations".to_string(),
        },
        // Future templates:
        // graphql, grpc, websocket, etc.
    ]
}
