use std::{fs::File, io::BufReader, sync::Arc};

use quinn::{Endpoint, SendStream};
use quinn_proto::ClientConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut BufReader::new(File::open(
        "certs/client.key.pem",
    )?))?;
    let key = rustls::PrivateKey(keys.remove(0));
    let ca_certs = rustls_pemfile::certs(&mut BufReader::new(File::open("certs/ca.pem")?))?;
    let client_certs = rustls_pemfile::certs(&mut BufReader::new(File::open("certs/client.pem")?))?;

    let crypto_config = quinn_test::tls::client(key, client_certs.into_iter().map(rustls::Certificate), ca_certs.into_iter().map(rustls::Certificate))?;

    let config = ClientConfig::new(Arc::new(crypto_config));

    let mut client = Endpoint::client("0.0.0.0:0".parse()?)?;
    client.set_default_client_config(config);

    let connecting = client.connect("127.0.0.1:4433".parse()?, "server.massive-linx")?;
    let connection = connecting.await?;
    println!("connection established {:?}", connection.rtt());
    
    let (mut send, mut recv) = connection.open_bi().await?;
    
    loop {
        let message = b"Ping from client!";
        send.write_all(message).await?;
        send.finish().await?;
        println!("Sent: {:?}", String::from_utf8_lossy(message));

        let mut buffer = vec![0; 1024];
        match recv.read(&mut buffer).await {
            Ok(Some(size)) => {
                let received_data = buffer[..size].to_vec();
                match String::from_utf8(received_data.clone()) {
                    Ok(string) => println!("Received String: {}", string),
                    Err(e) => println!("Failed to convert to string: {}", e),
                }
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

    println!("done!");
    Ok(())
}
