use saas::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


struct AppState {
    user_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_chat=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(100);

    let app_state = Arc::new(AppState {user_set, tx});

    let app = Router::new()
        .route("/", get(index))
        .route("/websocket", get(websocket_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    saas::serve(listener, app).await.unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();

    let mut username = String::new();

    // 先从输入留第一次获取到用户名称，一般是做链接认证
    while let Some(Ok(message)) = receiver.next().await {
        if let Message::Text(name) = message {
            check_username(&state, &mut username, &name);

            if !username.is_empty() {
                break;
            } else {
                let _ = sender
                    .send(Message::Text(String::from("username already toaken.")))
                    .await;

                return;
            }
        }
    }

    // 创建一个接受者
    let mut rx = state.tx.subscribe();

    let msg = format!("{} joined.", username);
    tracing::debug!("{}", msg);
    let _ = state.tx.send(msg);

    // 如果收到了通道的消息，则把他发送给websocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    let tx = state.tx.clone();
    let name = username.clone();

    // 如果收到了websocket的消息，则把他发送到通道中
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            let _ = tx.send(format!("{}: {}", name, text));
        }
    });

    
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    let msg = format!("{} left.", username);
    tracing::debug!("{}", msg);

    let _ = state.tx.send(msg);

    state.user_set.lock().unwrap().remove(&username);
}

fn check_username(state: &AppState, string: &mut String, name: &str) {
    let mut user_set = state.user_set.lock().unwrap();

    if !user_set.contains(name) {
        user_set.insert(name.to_owned());
        string.push_str(name);
    }
}

async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
