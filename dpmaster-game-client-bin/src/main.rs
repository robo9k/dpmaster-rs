use dpmaster_codec::GameClientCodec;
use dpmaster_proto::messages::{FilterOptions, GetServersMessage};
use futures::{FutureExt, SinkExt};
use std::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::udp::UdpFramed;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    println!("Bound socket to local address {:?}", socket.local_addr()?);

    let mut framed = UdpFramed::new(socket, GameClientCodec::new());

    let addr = "master.worldofpadman.com:27955"
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    println!("master server address: {:?}", addr);

    let getservers = GetServersMessage::new(
        Some(b"WorldofPadman".to_vec()),
        71,
        FilterOptions::new(None, true, true),
    );
    println!("Sending request {:?}", getservers);
    framed.send((getservers, addr)).await?;

    let (getserversresponse, _addr) = framed.next().map(|e| e.unwrap()).await?;
    println!("response: {:?}", getserversresponse);

    Ok(())
}
