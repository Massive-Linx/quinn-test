use std::{
    fs,
    fs::File,
    net::SocketAddr,
    sync::Arc,
    io::BufReader,
};

use quinn::{Endpoint, ServerConfig, TransportConfig};
use ::rustls::{Certificate, PrivateKey};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create and configure the transport configuration
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(
        "certs/server.key.pem",
    )?))?;
    let key = rustls::PrivateKey(keys.remove(0));
    let ca_certs = rustls_pemfile::certs(&mut BufReader::new(File::open("certs/ca.pem")?))?;
    let server_certs = rustls_pemfile::certs(&mut BufReader::new(File::open("certs/server.pem")?))?;

    let crypto_config = quinn_test::tls::server(
        key,
        server_certs.into_iter().map(rustls::Certificate),
        ca_certs.into_iter().map(rustls::Certificate),
    )?;

    let config = ServerConfig::with_crypto(Arc::new(crypto_config));

    let bind_addr: SocketAddr = "127.0.0.1:4433".parse()?;
    let server = Endpoint::server(config, bind_addr)?;

    println!("Server running on {}", bind_addr);

    while let Some(connecting) = server.accept().await {
        tokio::spawn(handle_connection(connecting));
    }

    Ok(())
}

async fn handle_connection(connecting: quinn::Connecting) {
    match connecting.await {
        Ok(connection) => {
            println!("Connection established: {:?}", connection.rtt());
            // Here you can handle incoming streams, etc.
            loop {
                match connection.accept_bi().await {
                    Ok((mut send, mut recv)) => {
                        // tokio::spawn(handle_stream(state.clone(), conn_state.clone(), send, recv))
                        let message = b"Ping from server!";
                        loop {
                            let mut buffer = vec![0; 1024];
                            match recv.read(&mut buffer).await {
                                Ok(Some(size)) => {
                                    let received_data = buffer[..size].to_vec();
                                    match String::from_utf8(received_data.clone()) {
                                        Ok(string) => println!("Received String: {}", string),
                                        Err(e) => println!("Failed to convert to string: {}", e),
                                    }
                                    send.write_all(message).await.expect("Failed to send");
                                    send.finish().await.expect("Failed to finish send");
                                },
                                Ok(None) => {
                                    println!("Stream closed");
                                    break;
                                }
                                Err(e) => {
                                    eprintln!("Failed to read: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        break;
                    }
                };
        
                println!("connection continue {:?}", connection.rtt());
            }
        }
        Err(e) => {
            eprintln!("Connection failed: {}", e);
        }
    }
}