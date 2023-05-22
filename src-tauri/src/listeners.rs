use tokio::net::{TcpListener, UnixListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[cfg(target_family = "unix")]
async fn create_listeners() {
    let tcp_handle = tokio::spawn(create_listeners_tcp_http());
    let unix_handle = tokio::spawn(create_listener_socket_unix());

    let _ = tokio::try_join!(tcp_handle, unix_handle);
}

#[cfg(target_family = "unix")]
async fn create_listener_socket_unix() {
    let listener = UnixListener::bind("/tmp/my_socket.sock").unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // In a loop, read data from the socket and process the requests.
            loop {
                match socket.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }

                        // Process the request...
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                }
            }
        });
    }
}

async fn create_listeners_tcp_http() {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // In a loop, read data from the socket and process the requests.
            loop {
                match socket.read(&mut buf).await {
                    Ok(n) => {
                        if n == 0 {
                            return;
                        }

                        // Process the request...
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                }
            }
        });
    }
}

