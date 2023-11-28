use utoipa::{
    openapi::{
        path::{OperationBuilder, Parameter, ParameterBuilder, ParameterIn},
        request_body::RequestBodyBuilder,
        ArrayBuilder, ComponentsBuilder, ContentBuilder, InfoBuilder, LicenseBuilder,
        ObjectBuilder, OpenApi, OpenApiBuilder, PathItem, PathItemType, PathsBuilder, Ref,
        Required, ResponseBuilder, ResponsesBuilder, SchemaType,
    },
    OpenApi as OpenApiTrait,
};

struct ApiDoc;

impl OpenApiTrait for ApiDoc {
    fn openapi() -> OpenApi {
        let mut openapi = OpenApiBuilder::new()
            .info(
                InfoBuilder::new()
                    .title("push_server")
                    .version("0.1.0")
                    .description(Some(""))
                    .license(Some(LicenseBuilder::new().name("").build())),
            )
            .paths(
                PathsBuilder::new()
                    .path(
                        "/health".to_string(),
                        PathItem::new(
                            PathItemType::Get,
                            OperationBuilder::new()
                                .responses(
                                    ResponsesBuilder::new()
                                        .response(
                                            "200",
                                            ResponseBuilder::new()
                                                .description("Server is healthy.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "ServerHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "405",
                                            ResponseBuilder::new()
                                                .description("Method not allowed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .build(),
                                )
                                .operation_id(Some("health"))
                                .summary(Some("Health check for the server"))
                                .description(Some("Health check for the server")),
                        ),
                    )
                    .path(
                        "/health_all".to_string(),
                        PathItem::new(
                            PathItemType::Get,
                            OperationBuilder::new()
                                .responses(
                                    ResponsesBuilder::new()
                                        .response(
                                            "200",
                                            ResponseBuilder::new()
                                                .description("All plugins are healthy.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PluginsHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "207",
                                            ResponseBuilder::new()
                                                .description("Some plugins are unhealthy.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PluginsHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "503",
                                            ResponseBuilder::new()
                                                .description("All plugins are unhealthy.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PluginsHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "405",
                                            ResponseBuilder::new()
                                                .description("Method not allowed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .build(),
                                )
                                .operation_id(Some("health_all"))
                                .summary(Some("Health check for all plugins"))
                                .description(Some("Health check for all plugins")),
                        ),
                    )
                    .path(
                        "/health_named/{plugin_name}".to_string(),
                        PathItem::new(
                            PathItemType::Get,
                            OperationBuilder::new()
                                .responses(
                                    ResponsesBuilder::new()
                                        .response(
                                            "200",
                                            ResponseBuilder::new()
                                                .description("Plugin is healthy.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PlugingHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "404",
                                            ResponseBuilder::new()
                                                .description("Plugin was not found.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PlugingHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "503",
                                            ResponseBuilder::new()
                                                .description("Plugin is unhealthy.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PlugingHealthResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Invalid path.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Missing path params.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Iternal server error.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "405",
                                            ResponseBuilder::new()
                                                .description("Method not allowed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .build(),
                                )
                                .operation_id(Some("health_named"))
                                .summary(Some("Health check for a specific plugin"))
                                .description(Some("Health check for a specific plugin"))
                                .parameter(
                                    ParameterBuilder::from(Parameter::new("plugin_name"))
                                        .parameter_in(ParameterIn::Path)
                                        .description(Some("Name of the plugin to check."))
                                        .schema(Some(
                                            ObjectBuilder::new().schema_type(SchemaType::String),
                                        ))
                                        .required(Required::True),
                                ),
                        ),
                    )
                    .path(
                        "/push".to_string(),
                        PathItem::new(
                            PathItemType::Post,
                            OperationBuilder::new()
                                .request_body(Some(
                                    RequestBodyBuilder::new()
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(Ref::from_schema_name("AlermanagerPush"))
                                                .build(),
                                        )
                                        .required(Some(Required::True))
                                        .build(),
                                ))
                                .responses(
                                    ResponsesBuilder::new()
                                        .response(
                                            "200",
                                            ResponseBuilder::new()
                                                .description("Push was successful.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "207",
                                            ResponseBuilder::new()
                                                .description("Some pushes were successful.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Push failed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "404",
                                            ResponseBuilder::new()
                                                .description("No plugins were found.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "422",
                                            ResponseBuilder::new()
                                                .description("Unprocessable Entity.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Invalid JSON.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Failed to buffer the request body.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "415",
                                            ResponseBuilder::new()
                                                .description(
                                                    "Unsupported media type. Header is missing.",
                                                )
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "413",
                                            ResponseBuilder::new()
                                                .description("Payload too large.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "405",
                                            ResponseBuilder::new()
                                                .description("Method not allowed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .build(),
                                )
                                .operation_id(Some("push"))
                                .summary(Some("Push alerts to all plugins asynchronously"))
                                .description(Some("Push alerts to all plugins asynchronously")),
                        ),
                    )
                    .path(
                        "/push_grouped/{plugin_group}".to_string(),
                        PathItem::new(
                            PathItemType::Post,
                            OperationBuilder::new()
                                .request_body(Some(
                                    RequestBodyBuilder::new()
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(Ref::from_schema_name("AlermanagerPush"))
                                                .build(),
                                        )
                                        .required(Some(Required::True))
                                        .build(),
                                ))
                                .responses(
                                    ResponsesBuilder::new()
                                        .response(
                                            "200",
                                            ResponseBuilder::new()
                                                .description("Push was successful.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "207",
                                            ResponseBuilder::new()
                                                .description("Some pushes were successful.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Push failed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "404",
                                            ResponseBuilder::new()
                                                .description("No plugins were found.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("PushResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Invalid path.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Missing path params.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Iternal server error.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "422",
                                            ResponseBuilder::new()
                                                .description("Unprocessable Entity.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Invalid JSON.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Failed to buffer the request body.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "415",
                                            ResponseBuilder::new()
                                                .description(
                                                    "Unsupported media type. Header is missing.",
                                                )
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "413",
                                            ResponseBuilder::new()
                                                .description("Payload too large.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "405",
                                            ResponseBuilder::new()
                                                .description("Method not allowed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .build(),
                                )
                                .operation_id(Some("push_grouped"))
                                .summary(Some("Push alerts to plugins in a group asynchronously"))
                                .description(Some(
                                    "Push alerts to plugins in a group asynchronously",
                                ))
                                .parameter(
                                    ParameterBuilder::from(Parameter::new("plugin_group"))
                                        .parameter_in(ParameterIn::Path)
                                        .description(Some("Name of the plugin group to push to."))
                                        .schema(Some(
                                            ObjectBuilder::new().schema_type(SchemaType::String),
                                        ))
                                        .required(Required::True),
                                ),
                        ),
                    )
                    .path(
                        "/push_named/{plugin_name}".to_string(),
                        PathItem::new(
                            PathItemType::Post,
                            OperationBuilder::new()
                                .request_body(Some(
                                    RequestBodyBuilder::new()
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(Ref::from_schema_name("AlermanagerPush"))
                                                .build(),
                                        )
                                        .required(Some(Required::True))
                                        .build(),
                                ))
                                .responses(
                                    ResponsesBuilder::new()
                                        .response(
                                            "200",
                                            ResponseBuilder::new()
                                                .description("Push was successful.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PluginPushResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "404",
                                            ResponseBuilder::new()
                                                .description("Plugin was not found.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PluginPushResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Push failed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name(
                                                                "PluginPushResponse",
                                                            ),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Invalid path.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Missing path params.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "500",
                                            ResponseBuilder::new()
                                                .description("Iternal server error.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "422",
                                            ResponseBuilder::new()
                                                .description("Unprocessable Entity.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Invalid JSON.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "400",
                                            ResponseBuilder::new()
                                                .description("Failed to buffer the request body.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "415",
                                            ResponseBuilder::new()
                                                .description(
                                                    "Unsupported media type. Header is missing.",
                                                )
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "413",
                                            ResponseBuilder::new()
                                                .description("Payload too large.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .response(
                                            "405",
                                            ResponseBuilder::new()
                                                .description("Method not allowed.")
                                                .content(
                                                    "application/json",
                                                    ContentBuilder::new()
                                                        .schema(ArrayBuilder::new().items(
                                                            Ref::from_schema_name("ErrorResponse"),
                                                        ))
                                                        .build(),
                                                )
                                                .build(),
                                        )
                                        .build(),
                                )
                                .operation_id(Some("push_named"))
                                .summary(Some("Push alerts to a specific plugin"))
                                .description(Some("Push alerts to a specific plugin"))
                                .parameter(
                                    ParameterBuilder::from(Parameter::new("plugin_name"))
                                        .parameter_in(ParameterIn::Path)
                                        .description(Some("Name of the plugin to push to."))
                                        .schema(Some(
                                            ObjectBuilder::new().schema_type(SchemaType::String),
                                        ))
                                        .required(Required::True),
                                ),
                        ),
                    ),
            )
            .components(Some(
                ComponentsBuilder::new()
                    .schema_from::<models::AlermanagerPush>()
                    .schema_from::<models::Status>()
                    .schema_from::<models::Alert>()
                    .schema_from::<crate::error_response::ErrorResponse>()
                    .schema_from::<crate::error_response::ErrorResponseType>()
                    .schema_from::<crate::error_response::PayloadInvalid>()
                    .schema_from::<crate::error_response::PathInvalid>()
                    .schema_from::<crate::error_response::InternalServerError>()
                    .schema_from::<crate::routes::push::PushStatus>()
                    .schema_from::<crate::routes::push::PluginPushStatus>()
                    .schema_from::<crate::routes::push::PluginPushResponse>()
                    .schema_from::<crate::routes::push::PushResponse>()
                    .schema_from::<crate::routes::health::ServerHealthResponse>()
                    .schema_from::<crate::routes::health::HealthStatus>()
                    .schema_from::<crate::routes::health::PluginHealthStatus>()
                    .schema_from::<crate::routes::health::PluginsHealthResponse>()
                    .schema_from::<crate::routes::health::PlugingHealthResponse>()
                    .build(),
            ))
            .tags(Some([]))
            .build();
        let _mods: [&dyn utoipa::Modify; 0usize] = [];
        _mods
            .iter()
            .for_each(|modifier| modifier.modify(&mut openapi));

        for (p_n, path_item) in openapi.paths.paths.iter_mut() {
            println!("path: {}", p_n);
            for (_, operation) in path_item.operations.iter_mut() {
                if let Some(ref mut parameters) = operation.parameters {
                    'parameters: for parameter in parameters.iter_mut() {
                        if let ParameterIn::Path = parameter.parameter_in {
                            println!(" Has path parameter");

                            operation.responses.responses.insert(
                                String::from("400"),
                                ResponseBuilder::new()
                                    .description("Invalid path.")
                                    .content(
                                        "application/json",
                                        ContentBuilder::new()
                                            .schema(
                                                ArrayBuilder::new()
                                                    .items(Ref::from_schema_name("ErrorResponse")),
                                            )
                                            .build(),
                                    )
                                    .build()
                                    .into(),
                            );
                            operation.responses.responses.insert(
                                String::from("500"),
                                ResponseBuilder::new()
                                    .description("Missing path params.")
                                    .content(
                                        "application/json",
                                        ContentBuilder::new()
                                            .schema(
                                                ArrayBuilder::new()
                                                    .items(Ref::from_schema_name("ErrorResponse")),
                                            )
                                            .build(),
                                    )
                                    .build()
                                    .into(),
                            );
                            operation.responses.responses.insert(
                                String::from("500"),
                                ResponseBuilder::new()
                                    .description("Iternal server error.")
                                    .content(
                                        "application/json",
                                        ContentBuilder::new()
                                            .schema(
                                                ArrayBuilder::new()
                                                    .items(Ref::from_schema_name("ErrorResponse")),
                                            )
                                            .build(),
                                    )
                                    .build()
                                    .into(),
                            );

                            break 'parameters;
                        }
                    }
                }

                if operation.request_body.is_some() {
                    println!(" Has request body");

                    operation.responses.responses.insert(
                        String::from("422"),
                        ResponseBuilder::new()
                            .description("Unprocessable Entity.")
                            .content(
                                "application/json",
                                ContentBuilder::new()
                                    .schema(
                                        ArrayBuilder::new()
                                            .items(Ref::from_schema_name("ErrorResponse")),
                                    )
                                    .build(),
                            )
                            .build()
                            .into(),
                    );

                    operation.responses.responses.insert(
                        String::from("400"),
                        ResponseBuilder::new()
                            .description("Invalid JSON.")
                            .content(
                                "application/json",
                                ContentBuilder::new()
                                    .schema(
                                        ArrayBuilder::new()
                                            .items(Ref::from_schema_name("ErrorResponse")),
                                    )
                                    .build(),
                            )
                            .build()
                            .into(),
                    );

                    operation.responses.responses.insert(
                        String::from("400"),
                        ResponseBuilder::new()
                            .description("Failed to buffer the request body.")
                            .content(
                                "application/json",
                                ContentBuilder::new()
                                    .schema(
                                        ArrayBuilder::new()
                                            .items(Ref::from_schema_name("ErrorResponse")),
                                    )
                                    .build(),
                            )
                            .build()
                            .into(),
                    );

                    operation.responses.responses.insert(
                        String::from("415"),
                        ResponseBuilder::new()
                            .description("Unsupported media type. Header is missing.")
                            .content(
                                "application/json",
                                ContentBuilder::new()
                                    .schema(
                                        ArrayBuilder::new()
                                            .items(Ref::from_schema_name("ErrorResponse")),
                                    )
                                    .build(),
                            )
                            .build()
                            .into(),
                    );

                    operation.responses.responses.insert(
                        String::from("413"),
                        ResponseBuilder::new()
                            .description("Payload too large.")
                            .content(
                                "application/json",
                                ContentBuilder::new()
                                    .schema(
                                        ArrayBuilder::new()
                                            .items(Ref::from_schema_name("ErrorResponse")),
                                    )
                                    .build(),
                            )
                            .build()
                            .into(),
                    );
                }

                operation.responses.responses.insert(
                    String::from("405"),
                    ResponseBuilder::new()
                        .description("Method not allowed.")
                        .content(
                            "application/json",
                            ContentBuilder::new()
                                .schema(
                                    ArrayBuilder::new()
                                        .items(Ref::from_schema_name("ErrorResponse")),
                                )
                                .build(),
                        )
                        .build()
                        .into(),
                );

                for (res_status, _) in operation.responses.responses.iter_mut() {
                    println!("  response: {}", res_status);
                }
            }
        }

        openapi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi() {
        ApiDoc::openapi();
    }
}
