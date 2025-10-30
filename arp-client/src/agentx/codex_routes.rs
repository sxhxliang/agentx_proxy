use crate::agentx::codex;
use crate::router::{HandlerContext, RouterBuilder};
use common::http;
use serde_json::json;

// Helper to validate and extract session_id from path parameters
fn get_session_id(ctx: &HandlerContext) -> Option<String> {
    ctx.path_params
        .get("session_id")
        .filter(|v| !v.is_empty())
        .map(|v| v.clone())
}

pub fn register_codex_project_routes(router_builder: &mut RouterBuilder) {
    router_builder.get("/api/codex/projects", |ctx| async move {
        let mut stream = ctx.stream;
        match codex::list_projects().await {
            Ok(projects) => {
                let body = json!({
                    "type": "projects",
                    "projects": projects
                });
                let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
            }
            Err(e) => {
                let _ = http::json_error(500, e).send(&mut stream).await;
            }
        }
        Ok(http::HttpResponse::ok())
    });

    router_builder.get(
        "/api/codex/projects/working-directories",
        |ctx| async move {
            let mut stream = ctx.stream;
            match codex::get_working_directories().await {
                Ok(directories) => {
                    let body = json!({
                        "directories": directories
                    });
                    let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
                }
                Err(e) => {
                    let _ = http::json_error(500, e).send(&mut stream).await;
                }
            }
            Ok(http::HttpResponse::ok())
        },
    );
}

pub fn register_codex_session_routes(router_builder: &mut RouterBuilder) {
    router_builder.get("/api/codex/sessions", |ctx| async move {
        let limit = ctx
            .request
            .query_param("limit")
            .and_then(|v| v.parse::<usize>().ok());
        let offset = ctx
            .request
            .query_param("offset")
            .and_then(|v| v.parse::<usize>().ok());
        let project_path = ctx.request.query_param("projectPath").cloned();

        let mut stream = ctx.stream;
        match codex::get_all_sessions(limit, offset, project_path).await {
            Ok(sessions) => {
                let body = json!({
                    "type": "sessions",
                    "sessions": sessions
                });
                let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
            }
            Err(e) => {
                let _ = http::json_error(500, e).send(&mut stream).await;
            }
        }
        Ok(http::HttpResponse::ok())
    });

    router_builder.get("/api/codex/sessions/{session_id}", |ctx| async move {
        let Some(session_id) = get_session_id(&ctx) else {
            let mut stream = ctx.stream;
            let _ = http::json_error(400, "session_id is required")
                .send(&mut stream)
                .await;
            return Ok(http::HttpResponse::ok());
        };

        let mut stream = ctx.stream;
        match codex::load_session_by_id(session_id.clone()).await {
            Ok(messages) => {
                let body = json!({
                    "type": "session_history",
                    "session_id": session_id,
                    "messages": messages
                });
                let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
            }
            Err(e) => {
                let _ = http::json_error(500, e).send(&mut stream).await;
            }
        }
        Ok(http::HttpResponse::ok())
    });

    router_builder.delete("/api/codex/sessions/{session_id}", |ctx| async move {
        let Some(session_id) = get_session_id(&ctx) else {
            let mut stream = ctx.stream;
            let _ = http::json_error(400, "session_id is required")
                .send(&mut stream)
                .await;
            return Ok(http::HttpResponse::ok());
        };

        let mut stream = ctx.stream;
        match codex::delete_session_by_id(session_id.clone()).await {
            Ok(_) => {
                let body = json!({
                    "type": "session_deleted",
                    "session_id": session_id
                });
                let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
            }
            Err(e) => {
                let status = if e.contains("not found") { 404 } else { 500 };
                let _ = http::json_error(status, e).send(&mut stream).await;
            }
        }
        Ok(http::HttpResponse::ok())
    });
}
