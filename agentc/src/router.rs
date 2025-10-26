use anyhow::Result;
use common::http::{HttpMethod, HttpRequest, HttpResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tracing::warn;

/// Handler context containing request and connection info
pub struct HandlerContext {
    pub request: HttpRequest,
    pub stream: TcpStream,
    pub proxy_conn_id: String,
    pub path_params: HashMap<String, String>,
}

/// Handler function type
pub type Handler = Arc<
    dyn Fn(
            HandlerContext,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<HttpResponse>> + Send>>
        + Send
        + Sync,
>;

/// Route definition
struct Route {
    method: Option<HttpMethod>,
    path_pattern: String,
    handler: Handler,
}

impl Route {
    fn matches(&self, method: &HttpMethod, path: &str) -> Option<HashMap<String, String>> {
        // Check method
        if let Some(ref route_method) = self.method {
            if route_method != method {
                return None;
            }
        }

        // Simple path matching (exact match or wildcard)
        if self.path_pattern == path {
            return Some(HashMap::new());
        }

        // Check for path parameters (e.g., /file/{path})
        let pattern_parts: Vec<&str> = self.path_pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();

        if pattern_parts.len() != path_parts.len() {
            return None;
        }

        let mut params = HashMap::new();

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with('{') && pattern_part.ends_with('}') {
                // Extract parameter name
                let param_name = &pattern_part[1..pattern_part.len() - 1];
                params.insert(param_name.to_string(), path_part.to_string());
            } else if pattern_part != path_part {
                return None;
            }
        }

        Some(params)
    }
}

/// HTTP router for handling requests
#[derive(Clone)]
pub struct Router {
    routes: std::sync::Arc<Vec<Route>>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Router {
            routes: std::sync::Arc::new(Vec::new()),
        }
    }

    /// Get a mutable reference to routes for building
    fn push_route(&mut self, route: Route) {
        std::sync::Arc::get_mut(&mut self.routes)
            .expect("Cannot modify router after it's been cloned")
            .push(route);
    }

    /// Add a route with any HTTP method
    pub fn route<F, Fut>(&mut self, path: impl Into<String>, handler: F)
    where
        F: Fn(HandlerContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<HttpResponse>> + Send + 'static,
    {
        let handler_arc = Arc::new(move |ctx: HandlerContext| {
            Box::pin(handler(ctx))
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<HttpResponse>> + Send>>
        });

        self.push_route(Route {
            method: None,
            path_pattern: path.into(),
            handler: handler_arc,
        });
    }

    /// Add a GET route
    pub fn get<F, Fut>(&mut self, path: impl Into<String>, handler: F)
    where
        F: Fn(HandlerContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<HttpResponse>> + Send + 'static,
    {
        let handler_arc = Arc::new(move |ctx: HandlerContext| {
            Box::pin(handler(ctx))
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<HttpResponse>> + Send>>
        });

        self.push_route(Route {
            method: Some(HttpMethod::GET),
            path_pattern: path.into(),
            handler: handler_arc,
        });
    }

    /// Add a POST route
    pub fn post<F, Fut>(&mut self, path: impl Into<String>, handler: F)
    where
        F: Fn(HandlerContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<HttpResponse>> + Send + 'static,
    {
        let handler_arc = Arc::new(move |ctx: HandlerContext| {
            Box::pin(handler(ctx))
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<HttpResponse>> + Send>>
        });

        self.push_route(Route {
            method: Some(HttpMethod::POST),
            path_pattern: path.into(),
            handler: handler_arc,
        });
    }

    /// Add a DELETE route
    pub fn delete<F, Fut>(&mut self, path: impl Into<String>, handler: F)
    where
        F: Fn(HandlerContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<HttpResponse>> + Send + 'static,
    {
        let handler_arc = Arc::new(move |ctx: HandlerContext| {
            Box::pin(handler(ctx))
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<HttpResponse>> + Send>>
        });

        self.push_route(Route {
            method: Some(HttpMethod::DELETE),
            path_pattern: path.into(),
            handler: handler_arc,
        });
    }

    /// Handle a request
    pub async fn handle(&self, mut ctx: HandlerContext) -> Result<HttpResponse> {
        // Handle OPTIONS requests for CORS preflight
        if ctx.request.method == HttpMethod::OPTIONS {
            return Ok(HttpResponse::new(204)
                .header("Access-Control-Allow-Origin", "*")
                .header(
                    "Access-Control-Allow-Methods",
                    "GET, POST, PUT, DELETE, PATCH, OPTIONS",
                )
                .header(
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization",
                )
                .header("Access-Control-Max-Age", "86400")
                .body(Vec::new()));
        }

        // Find matching route
        for route in self.routes.iter() {
            if let Some(params) = route.matches(&ctx.request.method, &ctx.request.path) {
                // Inject path parameters into context
                ctx.path_params = params;
                return (route.handler)(ctx).await;
            }
        }

        // No route found
        warn!(
            "No route found for {} {}",
            ctx.request.method.as_str(),
            ctx.request.path
        );
        Ok(HttpResponse::not_found().json(&serde_json::json!({
            "type": "error",
            "message": format!("Route not found: {} {}", ctx.request.method.as_str(), ctx.request.path)
        })))
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
