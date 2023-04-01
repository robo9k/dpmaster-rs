use clap::Parser as _;
use color_eyre::{eyre::Report, eyre::WrapErr};
use dpmaster_codec::GameClientCodec;
use dpmaster_proto::messages::{FilterOptions, GameName, GameType, GetServersMessage};
use eyre::eyre;
use futures::SinkExt;
use std::net::ToSocketAddrs;
use tokio::net::UdpSocket;
use tokio_stream::StreamExt;
use tokio_util::udp::UdpFramed;
use tracing::{debug, info};

/// Query dpmaster servers like a game client
#[derive(clap::Parser, Debug)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    /// Sends a `getservers` query
    GetServers(GetServersOpts),
}

// TODO: local_bind_addr
#[derive(clap::Parser, Debug)]
struct GetServersOpts {
    /// Address of the master server to query, e.g. `master.ioquake3.org:27950`
    #[arg(short, long)]
    master_server: String,

    /// Game name to query for, e.g. `Quake3Arena`
    #[arg(short = 'n', long)]
    game_name: Option<GameName>,

    /// Protocol version to query for, e.g. `68`
    #[arg(short, long)]
    protocol_number: u32,

    /// Game type to query for, e.g. `4` for CTF in Q3A
    #[arg(short = 't', long)]
    game_type: Option<GameType>,

    /// Ask for empty servers in query
    #[arg(short, long)]
    empty: bool,

    /// Ask for full servers in query
    #[arg(short, long)]
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
                getservers_opts.game_name,
                getservers_opts.protocol_number,
                FilterOptions::new(
                    getservers_opts.game_type,
                    getservers_opts.empty,
                    getservers_opts.full,
                ),
            );
            info!(request = ? getservers, "Sending request");
            framed.send((getservers, addr)).await?;

            while let Some((getserversresponse, _addr)) = framed
                .try_next()
                .await
                .wrap_err("Could not recieve message from master server")?
            {
                info!(response = ? getserversresponse, "Recieved message from master server");
                if getserversresponse.eot() {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_writer(std::io::stderr)
        .pretty();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
