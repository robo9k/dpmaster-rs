use clap::Clap;
use color_eyre::{eyre::Report, eyre::WrapErr};
use dpmaster_codec::GameClientCodec;
use dpmaster_proto::messages::{FilterOptions, GetServersMessage};
use eyre::eyre;
use futures::{FutureExt, SinkExt};
use std::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::udp::UdpFramed;
use tracing::{debug, info};

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
pub async fn main() -> Result<(), Report> {
    install_tracing();

    color_eyre::install()?;

    let opts: Opts = Opts::parse();
    debug!(?opts, "Parsed CLI options");

    match opts.subcmd {
        SubCommand::GetServers(getservers_opts) => {
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            info!(local_addr = % socket.local_addr()?, "Bound UDP socket");

            let mut framed = UdpFramed::new(socket, GameClientCodec::new());

            let addr = getservers_opts
                .master_server
                .to_socket_addrs()
                .wrap_err_with(|| {
                    format!(
                        "Failed to resolve master server {}",
                        getservers_opts.master_server
                    )
                })?
                .next()
                .ok_or_else(|| {
                    eyre!(
                        "Master server {} does not resolve to any address",
                        getservers_opts.master_server
                    )
                })?;
            info!(
                master_server = % addr,
                "Resolved master server {}", getservers_opts.master_server
            );

            let getservers = GetServersMessage::new(
                Some(getservers_opts.game_name),
                getservers_opts.protocol_number,
                FilterOptions::new(
                    getservers_opts.game_type,
                    getservers_opts.empty,
                    getservers_opts.full,
                ),
            );
            info!(request = ? getservers, "Sending request");
            framed.send((getservers, addr)).await?;

            let (getserversresponse, _addr) = framed.next().map(|e| e.unwrap()).await?; // TODO
            info!(response = ? getserversresponse, "Recieved response");
        }
    }

    Ok(())
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
