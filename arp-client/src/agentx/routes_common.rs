use crate::agentx::types::{Project, Session, WorkingDirectory};
use crate::router::RouterBuilder;
use common::http;
use serde_json::{Value, json};
use std::future::Future;

pub fn register_project_routes<ListProjectsFn, ListProjectsFut, WorkingDirsFn, WorkingDirsFut>(
    router_builder: &mut RouterBuilder,
    agent_name: &'static str,
    list_projects: ListProjectsFn,
    get_working_directories: WorkingDirsFn,
) where
    ListProjectsFn: Fn() -> ListProjectsFut + Send + Sync + 'static + Copy,
    ListProjectsFut: Future<Output = Result<Vec<Project>, String>> + Send + 'static,
    WorkingDirsFn: Fn() -> WorkingDirsFut + Send + Sync + 'static + Copy,
    WorkingDirsFut: Future<Output = Result<Vec<WorkingDirectory>, String>> + Send + 'static,
{
    router_builder.get(format!("/api/{}/projects", agent_name), move |ctx| {
        let list_projects_fn = list_projects;
        async move {
            let mut stream = ctx.stream;
            match list_projects_fn().await {
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
        }
    });

    router_builder.get(
        format!("/api/{}/projects/working-directories", agent_name),
        move |ctx| {
            let get_working_directories_fn = get_working_directories;
            async move {
                let mut stream = ctx.stream;
                match get_working_directories_fn().await {
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
            }
        },
    );
}

pub fn register_session_routes<
    GetSessionsFn,
    GetSessionsFut,
    LoadSessionFn,
    LoadSessionFut,
    DeleteSessionFn,
    DeleteSessionFut,
>(
    router_builder: &mut RouterBuilder,
    agent_name: &'static str,
    get_all_sessions: GetSessionsFn,
    load_session_by_id: LoadSessionFn,
    delete_session_by_id: DeleteSessionFn,
) where
    GetSessionsFn: Fn(Option<usize>, Option<usize>, Option<String>) -> GetSessionsFut
        + Send
        + Sync
        + 'static
        + Copy,
    GetSessionsFut: Future<Output = Result<Vec<Session>, String>> + Send + 'static,
    LoadSessionFn: Fn(String) -> LoadSessionFut + Send + Sync + 'static + Copy,
    LoadSessionFut: Future<Output = Result<Vec<Value>, String>> + Send + 'static,
    DeleteSessionFn: Fn(String) -> DeleteSessionFut + Send + Sync + 'static + Copy,
    DeleteSessionFut: Future<Output = Result<(), String>> + Send + 'static,
{
    router_builder.get(format!("/api/{}/sessions", agent_name), move |ctx| {
        let get_all_sessions_fn = get_all_sessions;
        async move {
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
            match get_all_sessions_fn(limit, offset, project_path).await {
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
        }
    });

    router_builder.get(
        format!("/api/{}/sessions/{{session_id}}", agent_name),
        move |ctx| {
            let load_session_by_id_fn = load_session_by_id;
            async move {
                let Some(session_id) = ctx
                    .path_params
                    .get("session_id")
                    .filter(|v| !v.is_empty())
                    .cloned()
                else {
                    let mut stream = ctx.stream;
                    let _ = http::json_error(400, "session_id is required")
                        .send(&mut stream)
                        .await;
                    return Ok(http::HttpResponse::ok());
                };

                let mut stream = ctx.stream;
                match load_session_by_id_fn(session_id.clone()).await {
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
            }
        },
    );

    router_builder.delete(
        format!("/api/{}/sessions/{{session_id}}", agent_name),
        move |ctx| {
            let delete_session_by_id_fn = delete_session_by_id;
            async move {
                let Some(session_id) = ctx
                    .path_params
                    .get("session_id")
                    .filter(|v| !v.is_empty())
                    .cloned()
                else {
                    let mut stream = ctx.stream;
                    let _ = http::json_error(400, "session_id is required")
                        .send(&mut stream)
                        .await;
                    return Ok(http::HttpResponse::ok());
                };

                let mut stream = ctx.stream;
                match delete_session_by_id_fn(session_id.clone()).await {
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
            }
        },
    );
}
