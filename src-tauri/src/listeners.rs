//listener.rs
use crate::state;
use axum;
use hyper::Server;
use hyperlocal::UnixServerExt;
use std::fs;
use tokio::sync::oneshot;

use axum::routing::Router;

// Bind the Unix socket

#[cfg(target_family = "windows")]
async fn create_listeners(app: Router) {}

async fn flatten<T>(handle: tokio::task::JoinHandle<Result<T, hyper::Error>>) -> Result<T, String> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err.to_string()),
        Err(_err) => Err("handling failed".into()),
    }
}

#[cfg(target_family = "unix")]
pub async fn create_listeners(
    app: Router<(), axum::body::Body>,
    rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    // Create an Axum `Server` using the Unix socket listener
    let (tx1, rx1) = oneshot::channel::<()>();
    let (tx2, rx2) = oneshot::channel::<()>();

    fs::remove_file("/tmp/pantrylocal.sock");
    let socket_fut = Server::bind_unix("/tmp/pantrylocal.sock")
        .map_err(|err| format!("failed to bind to socket: {:?}", err))?
        .serve(app.clone().into_make_service())
        .with_graceful_shutdown(async {
            rx1.await.ok();
        });

    let unix_handle = tokio::spawn(socket_fut);

    let tcp_fut = axum::Server::bind(&"0.0.0.0:9404".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            rx2.await.ok();
        });

    let tcp_handle = tokio::spawn(tcp_fut);

    tokio::spawn(async {
        rx.await;
        tx1.send(());
        tx2.send(());
    });

    match tokio::join!(flatten(tcp_handle), flatten(unix_handle)) {
        (Ok(_val), Ok(_val2)) => {
            println!("Success?!");
            fs::remove_file("/tmp/pantrylocal.sock")
                .map_err(|err| format!("Error removing file: {:?}", err))
        }
        (Err(err), Ok(_)) => {
            println!("Failed with {}.", err);
            Err(err)
        }
        (Ok(_), Err(err)) => {
            println!("Failed 2 with {}.", err);
            Err(err)
        }
        (Err(err), Err(err2)) => {
            println!("Failed double with {} and {}.", err, err2);
            Err(err)
        }
    }
    // if one crashes we DONT wanna die
    // match tokio::try_join!(flatten(tcp_handle), flatten(unix_handle)) {
    //     Ok(_val) => {
    //         println!("Success?!");
    //         fs::remove_file("/tmp/pantrylocal.sock")
    //             .map_err(|err| format!("Error removing file: {:?}", err))
    //     }
    //     Err(err) => {
    //         println!("Failed with {}.", err);
    //         Err(err)
    //     }
    // }
}

// On windows we only run the TCP one.
// There's an ongoing effort to implement window's new UDS support into tokio, but it's not ready
// yet and fixes currently out aren't super mature yet.
#[cfg(target_family = "windows")]
pub async fn create_listeners(
    app: Router<state::GlobalStateWrapper, axum::body::Body>,
    rx: oneshot::Receiver<()>,
) -> Result<(), String> {
    // Create an Axum `Server` using the Unix socket listener

    let tcp_fut = axum::Server::bind(&"0.0.0.0:9404".parse().unwrap())
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            rx.await.ok();
        });

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
