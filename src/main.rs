use std::{net::SocketAddr, time::Duration};

use clap::Parser;
use tokio::net::{TcpListener, TcpStream};
use anyhow::{bail, Result};
use tokio::time::sleep;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};

/// Simple program to greet a person
#[derive(Parser)]
#[command(version, about, long_about = None)]
enum Cli{
    Server(ServerArgs),
    Client(ClientArgs),
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ServerArgs{
    #[arg(short,long, default_value = "5000")]
    port: u16,
}

#[derive(clap::Args)]
#[command(version, about, long_about = None)]
struct ClientArgs{
    #[arg(short,long, default_value = "5000")]
    port: u16,

    #[arg(short,long, default_value = "127.0.0.1")]
    address: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match args{
        Cli::Server(args) => do_server(args).await?,
        Cli::Client(args) => do_client(args).await?,
    }

    Ok(())
}

async fn do_server(args: ServerArgs)->Result<()>{
    println!("Server running on port {}", args.port);

    let port = args.port;

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        
        println!("Got connection from {addr}");

        tokio::spawn(async move{
            let mut msg  = vec![0; 1024];
            loop {

                if let Err(e) = socket.readable().await {
                    println!("error: {e}");
                    break;
                };

                match socket.try_read(&mut msg ) {
                    Ok(n) => {
                        if n > 0 {
                            println!("received: {}", String::from_utf8_lossy(&msg [0..n]));
                        } else {
                            sleep(Duration::from_millis(10)).await;
                        }
                    }
                    Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                        continue;
                    }
                    Err(e) => {
                        println!("error: {e}");
                        break;
                    }
                }
            }

            println!("connection {addr} closed");
        });
    }
}

async fn do_client(args: ClientArgs)->Result<()>{
    println!("Client running on port {} with address {}", args.port, args.address);

    let address = args.address;
    let port = args.port;

    let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP))?;

    socket.set_keepalive(true)?;

    let address = format!("{address}:{port}").as_str().parse::<SocketAddr>()?;
    let address = address.into();
    socket.connect(&address)?;
    socket.set_nonblocking(true)?;
    let stream = TcpStream::from_std(socket.into())?;

    println!("connected to server");

    let mut count = 0u64;

    loop {
        count += 1;
        let payload = format!("hello world! {count}");

        stream.writable().await?;

        match stream.try_write(payload.as_bytes()) {
            Ok(_) => {
                println!("sent: {payload}");
            }
            Err(ref e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                println!("would block?");
            }
            Err(e) => {
                bail!("error: {e}")
            }
        }
        
        sleep(Duration::from_secs(1)).await;
    }
}
