use crate::handlers::{self, HandlerState};
use crate::router::Router;
use common::http;
use serde_json::json;

/// Build and return the router with all application routes registered.
pub fn build_router(state: HandlerState) -> Router {
    let mut router = Router::new();

    register_session_routes(&mut router, &state);
    register_claude_project_routes(&mut router);
    register_claude_session_routes(&mut router);
    register_proxy_routes(&mut router, &state);
    router
}

fn register_session_routes(router: &mut Router, state: &HandlerState) {
    // POST /sessions - Create new command execution session
    router.post("/sessions", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_session(ctx, state).await }
        }
    });

    // GET /sessions/{session_id} - Get session details or reconnect to active session
    router.get("/sessions/{session_id}", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_session(ctx, state).await }
        }
    });

    // DELETE /sessions/{session_id} - Cancel active session or delete historical session
    router.delete("/sessions/{session_id}", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_session(ctx, state).await }
        }
    });

    // POST /sessions/{session_id}/cancel - Cancel session without deleting history
    router.post("/sessions/{session_id}/cancel", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_cancel_session(ctx, state).await }
        }
    });
}

fn register_claude_project_routes(router: &mut Router) {
    router.get("/claude/projects", |ctx| async move {
        let mut stream = ctx.stream;
        match crate::claude::list_projects().await {
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

    router.get("/claude/projects/working-directories", |ctx| async move {
        let mut stream = ctx.stream;
        match crate::claude::get_working_directories().await {
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

    router.get("/claude/projects/{project_id}/sessions", |ctx| async move {
        let project_id = match ctx.path_params.get("project_id") {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                let mut stream = ctx.stream;
                let _ = http::json_error(400, "project_id is required")
                    .send(&mut stream)
                    .await;
                return Ok(http::HttpResponse::ok());
            }
        };

        let mut stream = ctx.stream;
        match crate::claude::get_project_sessions(project_id.clone()).await {
            Ok(sessions) => {
                let body = json!({
                    "type": "sessions",
                    "project_id": project_id,
                    "sessions": sessions
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
}

fn register_claude_session_routes(router: &mut Router) {
    router.get("/claude/sessions", |ctx| async move {
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
        match crate::claude::get_all_sessions(limit, offset, project_path).await {
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

    router.get("/claude/sessions/{session_id}", |ctx| async move {
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
        match crate::claude::load_session_by_id(session_id.clone()).await {
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

    router.delete("/claude/sessions/{session_id}", |ctx| async move {
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
        match crate::claude::delete_session_by_id(session_id.clone()).await {
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

fn register_proxy_routes(router: &mut Router, state: &HandlerState) {
    // Dynamic proxy route: /proxy/{port}/{*path}
    // This forwards requests to local services on different ports
    // Examples:
    //   /proxy/8080/api/users -> 127.0.0.1:8080/api/users
    //   /proxy/3000/ -> 127.0.0.1:3000/
    //   /proxy/9000/health?check=true -> 127.0.0.1:9000/health?check=true
    router.route("/proxy/{port}/{*path}", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::proxy::handle_dynamic_proxy(ctx, state).await }
        }
    });
}
