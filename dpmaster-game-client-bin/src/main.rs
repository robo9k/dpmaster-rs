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

    let addr = "master.ioquake3.org:27950"
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    framed
        .send((
            GetServersMessage::new(None, 68, FilterOptions::new(None, true, true)),
            addr,
        ))
        .await?;

    let (getserversresponse, addr) = framed.next().map(|e| e.unwrap()).await?;
    println!("response: {:?}", getserversresponse);

    Ok(())
}
