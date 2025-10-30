use crate::agentx::gemini;
use crate::agentx::routes_common::{register_project_routes, register_session_routes};
use crate::router::RouterBuilder;

pub fn register_gemini_project_routes(router_builder: &mut RouterBuilder) {
    register_project_routes(
        router_builder,
        "gemini",
        gemini::list_projects,
        gemini::get_working_directories,
    );
}

pub fn register_gemini_session_routes(router_builder: &mut RouterBuilder) {
    register_session_routes(
        router_builder,
        "gemini",
        gemini::get_all_sessions,
        gemini::load_session_by_id,
        gemini::delete_session_by_id,
    );
}
