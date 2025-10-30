use crate::agentx::claude_routes::{
    register_claude_project_routes, register_claude_session_routes,
};
use crate::agentx::codex_routes::{register_codex_project_routes, register_codex_session_routes};
use crate::handlers::{self, HandlerState};
use crate::router::{Router, RouterBuilder};

/// Build and return the router with all application routes registered.
pub fn build_router(state: HandlerState) -> Router {
    let mut builder = RouterBuilder::new();

    register_session_routes(&mut builder, &state);
    register_claude_project_routes(&mut builder);
    register_claude_session_routes(&mut builder);
    register_codex_project_routes(&mut builder);
    register_codex_session_routes(&mut builder);
    register_proxy_routes(&mut builder, &state);
    builder.build()
}

fn register_session_routes(router_builder: &mut RouterBuilder, state: &HandlerState) {
    // POST /api/sessions - Create new command execution session
    router_builder.post("/api/sessions", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_session(ctx, state).await }
        }
    });

    // GET /api/sessions/{session_id} - Get session details or reconnect to active session
    router_builder.get("/api/sessions/{session_id}", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_session(ctx, state).await }
        }
    });

    // DELETE /api/sessions/{session_id} - Cancel active session or delete historical session
    router_builder.delete("/api/sessions/{session_id}", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_session(ctx, state).await }
        }
    });

    // POST /api/sessions/{session_id}/cancel - Cancel session without deleting history
    router_builder.post("/api/sessions/{session_id}/cancel", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::session::handle_cancel_session(ctx, state).await }
        }
    });
}

fn register_proxy_routes(router_builder: &mut RouterBuilder, state: &HandlerState) {
    // Dynamic proxy route: /proxy/{port}/{*path}
    // This forwards requests to local services on different ports
    // Examples:
    //   /proxy/8080/api/users -> 127.0.0.1:8080/api/users
    //   /proxy/3000/ -> 127.0.0.1:3000/
    //   /proxy/9000/health?check=true -> 127.0.0.1:9000/health?check=true
    router_builder.route("/proxy/{port}/{*path}", {
        let state = state.clone();
        move |ctx| {
            let state = state.clone();
            async move { handlers::proxy::handle_dynamic_proxy(ctx, state).await }
        }
    });
}
