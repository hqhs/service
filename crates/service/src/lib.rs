use std::{
    path::Path,
    sync::{Arc, RwLock},
};

use axum::{
    extract::{Request, State},
    handler::HandlerWithoutStateExt,
    middleware::{from_fn_with_state, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service},
    Extension, Router, ServiceExt,
};
use hyper::StatusCode;
use tera::Tera;
use tower_http::services::ServeDir;

#[derive(Debug, Clone)]
pub enum AppError {
    Internal(Arc<anyhow::Error>),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut response = match self {
            AppError::Internal(ref a) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", a),
            )
                .into_response(),
        };
        response.extensions_mut().insert(self);
        response
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        AppError::Internal(Arc::new(err.into()))
    }
}

type Template = Result<Html<String>, AppError>;
type TemplateStorage = RwLock<Tera>;

pub struct ServerState {
    // pub diesel: DieselPool<diesel_async::
    templates: TemplateStorage,
}

impl ServerState {
    pub fn new(templates: Tera) -> Self {
        let templates = RwLock::new(templates);
        ServerState { templates }
    }

    pub fn render_template(
        &self,
        name: &str,
        context: &tera::Context,
    ) -> Result<String, AppError> {
        let borrowed = self.templates.try_read().unwrap();
        borrowed.render(name, context).map_err(|e| e.into())
    }

    #[allow(dead_code)]
    pub fn reload_templates(&self) -> Result<(), AppError> {
        #[cfg(feature = "reload")]
        {
            let mut borrowed = self.templates.try_write().unwrap();
            return borrowed.full_reload().map_err(|e| e.into());
        }
        #[cfg(not(feature = "reload"))]
        return Ok(());
    }
}

#[derive(Clone)]
struct Cx {
    server: Arc<ServerState>,
    request_id: String,
}

pub fn build_root_router<P: AsRef<Path>>(
    server: Arc<ServerState>,
    path: P,
) -> Router {
    let common_middleware =
        from_fn_with_state(server, common_request_cx_middleware);
    let serve_dir = ServeDir::new(path);

    Router::new()
        .route("/", get(root))
        .nest_service("/static", serve_dir)
        .route_layer(common_middleware)
}

async fn root(Extension(cx): Extension<Cx>) -> Template {
    let template_cx = tera::Context::new();
    let template = cx.server.render_template("home.jinja2", &template_cx)?;
    Ok(Html(template))
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

async fn common_request_cx_middleware(
    State(server): State<Arc<ServerState>>,
    mut request: Request,
    next: Next,
) -> Response {
    let request_id = String::new();
    let cx = Cx { server, request_id };
    request.extensions_mut().insert(cx);

    let response = next.run(request).await;

    response
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}
