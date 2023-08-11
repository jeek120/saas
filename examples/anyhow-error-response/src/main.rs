use saas::{
    Router,
    response::{IntoResponse, Response},
    routing::get, routing::post,
    http::StatusCode,
};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(handler));
    let app = app.route("/", post(handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3002")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    saas::serve(listener, app).await.unwrap();
}

async fn handler() -> Result<(), AppError> {
    try_handler()?;
    Ok(())
}

fn try_handler() -> Result<(), anyhow::Error> {
    anyhow::bail!("it failed!")
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
