use crate::agentx::claude;
use crate::agentx::routes_common::{register_project_routes, register_session_routes};
use crate::router::RouterBuilder;

pub fn register_claude_project_routes(router_builder: &mut RouterBuilder) {
    register_project_routes(
        router_builder,
        "claude",
        claude::list_projects,
        claude::get_working_directories,
    );
}

pub fn register_claude_session_routes(router_builder: &mut RouterBuilder) {
    register_session_routes(
        router_builder,
        "claude",
        claude::get_all_sessions,
        claude::load_session_by_id,
        claude::delete_session_by_id,
    );
}
