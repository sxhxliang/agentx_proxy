use crate::agentx::codex;
use crate::agentx::routes_common::{register_project_routes, register_session_routes};
use crate::router::RouterBuilder;

pub fn register_codex_project_routes(router_builder: &mut RouterBuilder) {
    register_project_routes(
        router_builder,
        "codex",
        codex::list_projects,
        codex::get_working_directories,
    );
}

pub fn register_codex_session_routes(router_builder: &mut RouterBuilder) {
    register_session_routes(
        router_builder,
        "codex",
        codex::get_all_sessions,
        codex::load_session_by_id,
        codex::delete_session_by_id,
    );
}
