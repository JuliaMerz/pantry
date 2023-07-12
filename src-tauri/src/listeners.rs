use hyper::{
    service::{make_service_fn, service_fn},
    Body, Response, Server,
};
use hyperlocal::UnixServerExt;
use tokio::net::{TcpListener, UnixListener};

use axum::routing::Router;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// Bind the Unix socket

#[cfg(target_family = "windows")]
async fn create_listeners(app: Router) {}

async fn flatten<T>(handle: tokio::task::JoinHandle<Result<T, hyper::Error>>) -> Result<T, String> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err.to_string()),
        Err(err) => Err("handling failed".into()),
    }
}

#[cfg(target_family = "unix")]
pub async fn create_listeners(app: Router) -> Result<(), String> {
    // Create an Axum `Server` using the Unix socket listener

    let socket_fut = Server::bind_unix("/tmp/hyperlocal.sock")
        .map_err(|err| format!("failed to bind to socket: {:?}", err))?
        .serve(app.clone().into_make_service());

    let unix_handle = tokio::spawn(socket_fut);

    let tcp_fut =
        axum::Server::bind(&"0.0.0.0:9404".parse().unwrap()).serve(app.into_make_service());

    let tcp_handle = tokio::spawn(tcp_fut);

    // if one crashes we wanna die
    match tokio::try_join!(flatten(tcp_handle), flatten(unix_handle)) {
        Ok(val) => {
            println!("Success?!");
        }
        Err(err) => {
            println!("Failed with {}.", err);
        }
    };
    Ok(())
}

// On windows we only run the TCP one.
// There's an ongoing effort to implement window's new UDS support into tokio, but it's not ready
// yet and fixes currently out aren't super mature yet.
#[cfg(target_family = "windows")]
pub async fn create_listeners(app: Router) -> Result<(), String> {
    // Create an Axum `Server` using the Unix socket listener

    let tcp_fut =
        axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service());

    let tcp_handle = tokio::spawn(tcp_fut);

    // if one crashes we wanna die
    match tokio::try_join!(flatten(tcp_handle)) {
        Ok(val) => {
            println!("Success?!");
        }
        Err(err) => {
            println!("Failed with {}.", err);
        }
    };
    Ok(())
}
