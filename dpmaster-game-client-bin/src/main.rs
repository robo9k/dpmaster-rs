use clap::Clap;
use dpmaster_codec::GameClientCodec;
use dpmaster_proto::messages::{FilterOptions, GetServersMessage};
use futures::{FutureExt, SinkExt};
use std::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::udp::UdpFramed;

#[derive(Clap, Debug)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap, Debug)]
enum SubCommand {
    GetServers(GetServersOpts),
}

type Bytes = Vec<u8>;

// TODO: local_bind_addr
#[derive(Clap, Debug)]
struct GetServersOpts {
    #[clap(short, long)]
    master_server: String,

    #[clap(short = 'n', long, parse(from_str))]
    game_name: Bytes,

    #[clap(short, long)]
    protocol_number: u32,

    #[clap(short = 't', long, parse(from_str))]
    game_type: Option<Bytes>,

    #[clap(short, long)]
    empty: bool,

    #[clap(short, long)]
    full: bool,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    println!("opts: {:?}", opts);

    match opts.subcmd {
        SubCommand::GetServers(getservers_opts) => {
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            println!("Bound socket to local address {:?}", socket.local_addr()?);

            let mut framed = UdpFramed::new(socket, GameClientCodec::new());

            let addr = getservers_opts
                .master_server
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap();
            println!("master server address: {:?}", addr);

            let getservers = GetServersMessage::new(
                Some(getservers_opts.game_name),
                getservers_opts.protocol_number,
                FilterOptions::new(
                    getservers_opts.game_type,
                    getservers_opts.empty,
                    getservers_opts.full,
                ),
            );
            println!("Sending request {:?}", getservers);
            framed.send((getservers, addr)).await?;

            let (getserversresponse, _addr) = framed.next().map(|e| e.unwrap()).await?;
            println!("response: {:?}", getserversresponse);
        }
    }

    Ok(())
}
