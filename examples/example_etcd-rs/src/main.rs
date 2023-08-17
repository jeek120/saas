use etcd_rs::{Client, ClientConfig};

#[tokio::main]
async fn main() {
    let cli = Client::connect(ClientConfig {
        endpoints: vec![
            "http://127.0.0.1:2379".into(),
        ],
        ..Default::default()
    }).await?;

    cli.put(("foo", "bar")).await.expect("put kv");

    let kvs = cli.get("foo").await.expect("get kv").take_kvs();

    assert_eq!(kvs.len(), 1);
}
