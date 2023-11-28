use utoipa::{
    openapi::{
        path::ParameterIn, ArrayBuilder, ContentBuilder, OpenApi as OpenApiDoc, Ref, RefOr,
        ResponseBuilder,
    },
    OpenApi,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health,
        crate::routes::health::health_all,
        crate::routes::health::health_named,
        crate::routes::push::push,
        crate::routes::push::push_grouped,
        crate::routes::push::push_named
    ),
    components(schemas(
        models::AlermanagerPush,
        models::Status,
        models::Alert,
        crate::error_response::ErrorResponse,
        crate::error_response::ErrorResponseType,
        crate::error_response::PayloadInvalid,
        crate::error_response::PathInvalid,
        crate::error_response::InternalServerError,
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
                if let Some(ref mut parameters) = operation.parameters {
                    'parameters: for parameter in parameters.iter_mut() {
                        if let ParameterIn::Path = parameter.parameter_in {
                            operation
                                .responses
                                .responses
                                .entry(String::from("400"))
                                .and_modify(|existing_response| {
                                    if let RefOr::T(existing_response) = existing_response {
                                        existing_response.description.push_str(" Invalid path.");
                                    }
                                })
                                .or_insert(
                                    ResponseBuilder::new()
                                        .description("Invalid path.")
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(
                                                    ArrayBuilder::new().items(
                                                        Ref::from_schema_name("ErrorResponse"),
                                                    ),
                                                )
                                                .build(),
                                        )
                                        .build()
                                        .into(),
                                );
                            operation
                                .responses
                                .responses
                                .entry(String::from("500"))
                                .and_modify(|existing_response| {
                                    if let RefOr::T(existing_response) = existing_response {
                                        existing_response
                                            .description
                                            .push_str(" Missing path params.");
                                    }
                                })
                                .or_insert(
                                    ResponseBuilder::new()
                                        .description("Missing path params.")
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(
                                                    ArrayBuilder::new().items(
                                                        Ref::from_schema_name("ErrorResponse"),
                                                    ),
                                                )
                                                .build(),
                                        )
                                        .build()
                                        .into(),
                                );
                            operation
                                .responses
                                .responses
                                .entry(String::from("500"))
                                .and_modify(|existing_response| {
                                    if let RefOr::T(existing_response) = existing_response {
                                        existing_response
                                            .description
                                            .push_str(" Iternal server error.");
                                    }
                                })
                                .or_insert(
                                    ResponseBuilder::new()
                                        .description("Iternal server error.")
                                        .content(
                                            "application/json",
                                            ContentBuilder::new()
                                                .schema(
                                                    ArrayBuilder::new().items(
                                                        Ref::from_schema_name("ErrorResponse"),
                                                    ),
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
                    operation
                        .responses
                        .responses
                        .entry(String::from("422"))
                        .and_modify(|existing_response| {
                            if let RefOr::T(existing_response) = existing_response {
                                existing_response
                                    .description
                                    .push_str(" Unprocessable Entity.");
                            }
                        })
                        .or_insert(
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

                    operation
                        .responses
                        .responses
                        .entry(String::from("400"))
                        .and_modify(|existing_response| {
                            if let RefOr::T(existing_response) = existing_response {
                                existing_response.description.push_str(" Invalid JSON.");
                            }
                        })
                        .or_insert(
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

                    operation
                        .responses
                        .responses
                        .entry(String::from("400"))
                        .and_modify(|existing_response| {
                            if let RefOr::T(existing_response) = existing_response {
                                existing_response
                                    .description
                                    .push_str(" Failed to buffer the request body.");
                            }
                        })
                        .or_insert(
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

                    operation
                        .responses
                        .responses
                        .entry(String::from("415"))
                        .and_modify(|existing_response| {
                            if let RefOr::T(existing_response) = existing_response {
                                existing_response
                                    .description
                                    .push_str(" Unsupported media type: Header is missing.");
                            }
                        })
                        .or_insert(
                            ResponseBuilder::new()
                                .description("Unsupported media type: Header is missing.")
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

                    operation
                        .responses
                        .responses
                        .entry(String::from("413"))
                        .and_modify(|existing_response| {
                            if let RefOr::T(existing_response) = existing_response {
                                existing_response
                                    .description
                                    .push_str(" Payload too large.");
                            }
                        })
                        .or_insert(
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

                operation
                    .responses
                    .responses
                    .entry(String::from("405"))
                    .and_modify(|existing_response| {
                        if let RefOr::T(existing_response) = existing_response {
                            existing_response
                                .description
                                .push_str(" Method not allowed.");
                        }
                    })
                    .or_insert(
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
            }
        }

        openapi
    }
}
