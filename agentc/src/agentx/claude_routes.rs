use crate::agentx::claude;
use crate::router::RouterBuilder;
use common::http;
use serde_json::json;

pub fn register_claude_project_routes(router_builder: &mut RouterBuilder) {
    router_builder.get("/claude/projects", |ctx| async move {
        let mut stream = ctx.stream;
        match claude::list_projects().await {
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

    router_builder.get("/claude/projects/working-directories", |ctx| async move {
        let mut stream = ctx.stream;
        match claude::get_working_directories().await {
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
    });
}

pub fn register_claude_session_routes(router_builder: &mut RouterBuilder) {
    router_builder.get("/claude/sessions", |ctx| async move {
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
        match claude::get_all_sessions(limit, offset, project_path).await {
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

    router_builder.get("/claude/sessions/{session_id}", |ctx| async move {
        let session_id = match ctx.path_params.get("session_id") {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                let mut stream = ctx.stream;
                let _ = http::json_error(400, "session_id is required")
                    .send(&mut stream)
                    .await;
                return Ok(http::HttpResponse::ok());
            }
        };

        let mut stream = ctx.stream;
        match claude::load_session_by_id(session_id.clone()).await {
            Ok(messages) => {
                let body = json!({
                    "type": "session_history",
                    "session_id": session_id,
                    "messages": messages
                });
                let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
                Ok(http::HttpResponse::ok())
            }
            Err(e) => {
                let _ = http::json_error(500, e).send(&mut stream).await;
                Ok(http::HttpResponse::ok())
            }
        }
    });

    router_builder.delete("/claude/sessions/{session_id}", |ctx| async move {
        let session_id = match ctx.path_params.get("session_id") {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                let mut stream = ctx.stream;
                let _ = http::json_error(400, "session_id is required")
                    .send(&mut stream)
                    .await;
                return Ok(http::HttpResponse::ok());
            }
        };

        let mut stream = ctx.stream;
        match claude::delete_session_by_id(session_id.clone()).await {
            Ok(_) => {
                let body = json!({
                    "type": "session_deleted",
                    "session_id": session_id
                });
                let _ = http::HttpResponse::ok().json(&body).send(&mut stream).await;
                Ok(http::HttpResponse::ok())
            }
            Err(e) => {
                let status = if e.contains("not found") { 404 } else { 500 };
                let _ = http::json_error(status, e).send(&mut stream).await;
                Ok(http::HttpResponse::ok())
            }
        }
    });
}
