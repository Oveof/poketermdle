use futures_util::{SinkExt, StreamExt};
use log::debug;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;

async fn server(port: &str) {
    let server = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    while let Ok((stream, _)) = server.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let mut ws_stream = accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if msg.is_text() || msg.is_binary() {
            debug!("Server on message: {:?}", &msg);
            ws_stream.send(msg).await.unwrap();
        }
    }
}
