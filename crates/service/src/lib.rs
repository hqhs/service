use std::{
    fmt::Write,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    task::{Context, Poll},
};

use axum::{
    extract::Request,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use diesel::{
    r2d2::{self, ConnectionManager},
    SqliteConnection,
};
use futures::future::BoxFuture;
use hyper::StatusCode;
use serde::Serialize;
use tera::Tera;
use tower::{Layer, Service, ServiceBuilder};
use tower_http::services::ServeDir;
use uuid::Uuid;

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

#[derive(Clone, Serialize)]
struct UserSession {}

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
type Template = Result<Html<String>, AppError>;
type TemplateStorage = RwLock<Tera>;

pub struct ServerState {
    pub db_pool: DbPool,

    templates: TemplateStorage,
    dev_mode: bool,
}

impl ServerState {
    pub fn new(database_url: &str) -> anyhow::Result<Self> {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let db_pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        let templates = {
            let path: PathBuf =
                [env!("CARGO_MANIFEST_DIR"), "templates"].iter().collect();
            if !path.is_dir() {
                anyhow::bail!(
                    "{} directory does not exist",
                    path.to_string_lossy()
                );
            }
            let glob = format!("{}/**/*.jinja2", path.to_string_lossy());
            let mut templates = Tera::new(&glob)?;

            templates.autoescape_on(vec![".jinja2"]);
            RwLock::new(templates)
        };
        Ok(ServerState { db_pool, templates, dev_mode: true })
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
    user: Option<UserSession>,
}

impl Cx {
    fn render<T>(&self, template: &str, other: T) -> Result<String, AppError>
    where
        T: Serialize,
    {
        #[derive(Serialize)]
        struct Common<'a, T>
        where
            T: Serialize,
        {
            user: Option<&'a UserSession>,
            dev_mode: bool,
            request_id: String,

            #[serde(flatten)]
            other: T,
        }

        let cx = &self;
        let dev_mode = true;
        let request_id = cx.request_id.clone();
        let user = cx.user.as_ref();
        let params = Common { user, dev_mode, request_id, other };

        let template_cx = tera::Context::from_serialize(params)?;
        let unlocked = self.server.templates.read().unwrap();
        let maybe_page = unlocked.render(template, &template_cx);
        if let Err(ref err) = maybe_page {
            // NOTE: it's fine to log error internally since templates
            // are expected to always work, and all rendering happens threw
            // this method
            tracing::error!("failed to render {template}: {:#?}", err);
        }
        let page = maybe_page?;
        Ok(page)
    }
}

pub fn setup_app(database_url: &str) -> anyhow::Result<Router> {
    let server = Arc::new(ServerState::new(database_url)?);

    let static_path: PathBuf =
        [env!("CARGO_MANIFEST_DIR"), "static"].iter().collect();
    Ok(build_root_router(server, static_path))
}

fn build_root_router<P: AsRef<Path>>(
    server: Arc<ServerState>,
    path: P,
) -> Router {
    let serve_dir = ServeDir::new(path);

    let root_router = apply_middleware(
        Router::new().route("/", get(root)).nest_service("/static", serve_dir),
        server.clone(),
    );

    let debug_routes = apply_middleware(
        Router::new().route("/reload", post(reload_templates)),
        server,
    );

    Router::new().merge(root_router).merge(debug_routes)
}

async fn root(Extension(cx): Extension<Cx>) -> Template {
    let template = cx.render("home.jinja2", serde_json::Value::Null)?;
    Ok(Html(template))
}

async fn reload_templates(
    Extension(cx): Extension<Cx>,
) -> Result<StatusCode, AppError> {
    #[cfg(feature = "reload")]
    {
        tracing::info!("reloading templates...");
        cx.server.reload_templates()?;
    }
    Ok(StatusCode::OK)
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}

fn apply_middleware(router: Router, server: Arc<ServerState>) -> Router {
    // TODO:
    // https://docs.rs/tower/0.4.13/tower/index.html
    // - timeout layer
    // - buffer
    // - shed
    // etc.
    let layers =
        ServiceBuilder::new().layer(CommonRequestContextLayer { server });

    router.route_layer(layers)
}

/// CommonRequestContextLayer: TODO: explanation.
/// "Inspired by" https://docs.rs/axum/latest/axum/middleware/index.html#towerservice-and-pinboxdyn-future
#[derive(Clone)]
struct CommonRequestContextLayer {
    pub server: Arc<ServerState>,
}

impl<S> Layer<S> for CommonRequestContextLayer {
    type Service = CommonRequestContextService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CommonRequestContextService { inner, server: self.server.clone() }
    }
}

#[derive(Clone)]
struct CommonRequestContextService<S> {
    inner: S,
    server: Arc<ServerState>,
}

impl<S> Service<Request> for CommonRequestContextService<S>
where
    S: Service<Request, Response = Response> + Send + 'static + Clone,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut request: Request) -> Self::Future {
        // let future = self.inner.call(request);
        let clone = self.inner.clone();
        // NOTE: can't just clone inner service, explanation:
        // https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let server = self.server.clone();
        Box::pin(async move {
            let request_id = Uuid::new_v4().to_string();
            // let mut log = server.root.new(o!(
            //     "request_id" => request_id.clone(),
            //     "uri" => request.uri().to_string(),
            //     "method" => request.method().as_str().to_owned(),
            // ));
            // let jar = CookieJar::from_headers(request.headers());
            // let user = user_session_from_cookies(&jar, &server).await.ok();
            // if let Some(ref user) = user
            // {
            //     log = log.new(o!(
            //         "session_id" => user.session_id.to_string().clone(),
            //     ));
            // }
            let user = None;
            let cx = Cx { server, user, request_id };

            request.extensions_mut().insert(cx.clone());
            let response = inner.call(request).await?;
            let status = response.status();
            if status == StatusCode::INTERNAL_SERVER_ERROR {
                // TODO: slog::error! definetely misses here somewhere
                if let Some(err) = response.extensions().get::<AppError>() {
                    // NOTE: try to render 500.jinja2 with debug info in case
                    // it's template rendering error (probable)
                    let new_response: Response =
                        maybe_render_debug_info(&cx, err).into_response();
                    return Ok(new_response);
                }
            }
            Ok(response)
        })
    }
}

fn maybe_render_debug_info(cx: &Cx, err: &AppError) -> impl IntoResponse {
    #[derive(Serialize)]
    struct DebugInfoPayload {
        error: String,
    }

    let mut payload = DebugInfoPayload { error: String::new() };
    if cx.server.dev_mode {
        write!(&mut payload.error, "{:?}", err);
    } else {
        write!(&mut payload.error, "No debug info available.");
    }
    // https://postsrc.com/components/tailwind-css-error-pages/simple-500-error-component
    let page = match cx.render("500.jinja2", payload) {
        Ok(page) => page,
        Err(e) => {
            // NOTE: well in that case we're kinda fucked anyway
            if cx.server.dev_mode {
                format!("Failed to render `500.jinja2`: {:#?} to display another error: {:#?}", e, err)
            } else {
                "Internal server error; Something went terribly wrong! Please contant the site's administrators if you can.".to_string()
            }
        }
    };
    Html(page)
}
