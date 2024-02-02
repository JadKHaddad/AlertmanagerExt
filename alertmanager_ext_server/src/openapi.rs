use utoipa::{
    openapi::{
        path::{Operation, ParameterIn},
        ArrayBuilder, ContentBuilder, OpenApi as OpenApiDoc, Ref, RefOr, ResponseBuilder,
    },
    OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::metrics::metrics,
        crate::routes::health::health,
        crate::routes::health::plugin_health,
        crate::routes::push::push,
    ),
    components(schemas(
        models::AlertmanagerPush,
        models::Status,
        models::Alert,
        crate::error_response::ErrorResponse,
        crate::error_response::ErrorResponseType,
        crate::error_response::PayloadInvalid,
        crate::error_response::QueryInvalid,
        crate::error_response::PluginFilterInvalid,
        crate::error_response::PathInvalid,
        crate::error_response::InternalServerError,
        crate::routes::models::PluginResponseMeta,
        crate::routes::models::PluginFilterQuery,
        crate::routes::push::PushStatus,
        crate::routes::push::PluginPushStatus,
        crate::routes::push::PluginPushResponse,
        crate::routes::push::PushResponse,
        crate::routes::health::ServerHealthResponse,
        crate::routes::health::HealthStatus,
        crate::routes::health::PluginHealthStatus,
        crate::routes::health::PluginsHealthResponse,
        crate::routes::health::PlugingHealthResponse
    )),
    tags()
)]
struct ApiDoc;

pub struct OpenApiDocFinalizer;

impl OpenApi for OpenApiDocFinalizer {
    fn openapi() -> OpenApiDoc {
        let mut openapi = ApiDoc::openapi();

        for (_, path_item) in openapi.paths.paths.iter_mut() {
            for (_, operation) in path_item.operations.iter_mut() {
                if let Some(parameters) = operation.parameters.clone() {
                    // Errors returned by ApiQuery extractor
                    'query: for parameter in parameters.iter() {
                        if let ParameterIn::Query = parameter.parameter_in {
                            add_error_response(
                                operation,
                                String::from("400"),
                                "Failed to deserialize query string.",
                            );

                            break 'query;
                        }
                    }

                    // Errors returned by ApiPath extractor
                    'path: for parameter in parameters.iter() {
                        if let ParameterIn::Path = parameter.parameter_in {
                            add_error_response(operation, String::from("400"), "Invalid path.");
                            add_error_response(
                                operation,
                                String::from("500"),
                                "Missing path params.",
                            );
                            add_error_response(
                                operation,
                                String::from("500"),
                                "Iternal server error.",
                            );

                            break 'path;
                        }
                    }
                }

                // Errors returned by ApiJson extractor
                if operation.request_body.is_some() {
                    add_error_response(operation, String::from("422"), "Unprocessable Entity.");
                    add_error_response(operation, String::from("400"), "Invalid JSON.");
                    add_error_response(
                        operation,
                        String::from("400"),
                        "Failed to buffer the request body.",
                    );
                    add_error_response(
                        operation,
                        String::from("415"),
                        "Unsupported media type: Header is missing.",
                    );
                    add_error_response(operation, String::from("413"), "Payload too large.");
                }

                // Error returned by our custom middleware
                add_error_response(operation, String::from("405"), "Method not allowed.");

                // Error returned by ApiPluginFilterQuery extractor
                add_error_response(operation, String::from("422"), "Invalid filter.");
            }
        }

        openapi
    }
}

/// Adds default error responses to an operation
///
/// Errors returned by our extractors, middlewares and not found handler.
fn add_error_response(operation: &mut Operation, status: String, description: &str) {
    operation
        .responses
        .responses
        .entry(status)
        .and_modify(|existing_response| {
            if let RefOr::T(existing_response) = existing_response {
                existing_response
                    .description
                    .push_str(&format!(" {description}"));
            }
        })
        .or_insert(
            ResponseBuilder::new()
                .description(description)
                .content(
                    "application/json",
                    ContentBuilder::new()
                        .schema(ArrayBuilder::new().items(Ref::from_schema_name("ErrorResponse")))
                        .build(),
                )
                .build()
                .into(),
        );
}
