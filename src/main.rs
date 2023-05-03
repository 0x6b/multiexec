use std::{
    error::Error,
    fs::File,
    io::{BufReader, Read},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
    str::FromStr,
    time::Duration,
};

use dirs::home_dir;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ssh2::Session;
use ssh2_config::{HostParams, SshConfig};
use structopt::StructOpt;

use crate::node::Node;

mod node;

static TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

#[derive(Debug, StructOpt)]
struct Args {
    /// Command to execute.
    command: String,

    /// Path to ssh config file. Defaults to "~/.ssh/config".
    #[structopt(short, long)]
    ssh_config_path: Option<String>,

    /// Interval in seconds to execute the command. Defaults to 10.
    #[structopt(short, long, default_value = "10")]
    interval: u64,

    /// Comma separated list of nodes to execute the command on. Node can be specified by "node1" or "1".
    #[structopt(short, long, value_delimiter = ",", default_value = "1,2,3,4")]
    nodes: Vec<Node>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::from_args();
    let ssh_config = get_ssh_config(args.ssh_config_path)?;

    let mut tasks = Vec::new();
    let m = MultiProgress::new();

    args.nodes.into_iter().for_each(|node| {
        let pb = m.add(ProgressBar::new(1));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.bold} {prefix:.blue}: {wide_msg}")
                .unwrap()
                .tick_chars(TICK_CHARS),
        );
        pb.set_prefix(node.to_string());

        tasks.push(tokio::spawn(exec(
            args.command.clone(),
            ssh_config.query(node),
            args.interval,
            pb,
        )));
    });

    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}

async fn exec(command: String, host_params: HostParams, interval: u64, pb: ProgressBar) {
    let socket_addr = get_socket_addr(&host_params).unwrap();
    let user = host_params.user.clone().unwrap_or("root".to_string());
    let identity_file = get_first_identity_file(&host_params).unwrap();
    let mut interval = tokio::time::interval(Duration::from_millis(interval * 1000));

    for _ in 0.. {
        interval.tick().await;
        let now = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

        let stream = match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(10)) {
            Ok(s) => s,
            Err(e) => {
                pb.set_message(format!("{now} - Failed to connect: {}", e));
                pb.inc(1);
                continue;
            }
        };

        if let Err(e) = stream.set_read_timeout(Some(Duration::from_secs(10))) {
            pb.set_message(format!("{now} - Failed to set read timeout: {}", e));
            pb.inc(1);
            continue;
        }

        if let Err(e) = stream.set_write_timeout(Some(Duration::from_secs(10))) {
            pb.set_message(format!("{now} - Failed to set write timeout: {}", e));
            pb.inc(1);
            continue;
        }

        let mut sess = match Session::new() {
            Ok(s) => s,
            Err(e) => {
                pb.set_message(format!("{now} - Failed to create session: {}", e));
                pb.inc(1);
                continue;
            }
        };

        sess.set_tcp_stream(stream);
        sess.set_timeout(10 * 1000);

        if let Err(e) = sess.handshake() {
            pb.set_message(format!("{now} - Failed to handshake: {e}"));
            pb.inc(1);
            continue;
        }

        if let Err(e) = sess.userauth_pubkey_file(&user, None, &identity_file, None) {
            pb.set_message(format!("{now} - Failed to authenticate: {}", e));
            pb.inc(1);
            continue;
        }

        let mut channel = match sess.channel_session() {
            Ok(c) => c,
            Err(e) => {
                pb.set_message(format!("{now} - Failed to create channel: {}", e));
                pb.inc(1);
                continue;
            }
        };

        if let Err(e) = channel.exec(&command) {
            pb.set_message(format!("{now} - Failed to execute command: {}", e));
            pb.inc(1);
            continue;
        }

        let mut s = String::new();
        if let Err(e) = channel.read_to_string(&mut s) {
            pb.set_message(format!("{now} - Failed to read command output: {}", e));
            pb.inc(1);
            continue;
        };

        let result = s
            .lines()
            .map(|line| format!("{now} - {}", line))
            .collect::<Vec<String>>()
            .join("\n");

        pb.set_message(result);
        pb.inc(1);

        channel.send_eof().unwrap_or(()); // Ignore errors
        channel.wait_close().unwrap_or(()); // Ignore errors
        sess.disconnect(None, "disconnect", None).unwrap_or(()); // Ignore errors
    }
}

fn get_ssh_config(ssh_config_path: Option<String>) -> Result<SshConfig, Box<dyn Error>> {
    let ssh_config_path = match ssh_config_path {
        None => home_dir()
            .expect("Failed to determine home directory")
            .join(".ssh")
            .join("config"),
        Some(p) => p.into(),
    };
    let mut reader = BufReader::new(File::open(&ssh_config_path)?);
    let ssh_config = SshConfig::default().parse(&mut reader).unwrap_or_else(|_| {
        panic!(
            "Failed to parse configuration: {}",
            &ssh_config_path.display()
        )
    });

    Ok(ssh_config)
}

fn get_socket_addr(host_params: &HostParams) -> Result<SocketAddr, Box<dyn Error>> {
    let host_name = host_params
        .host_name
        .as_ref()
        .expect("hostname is required");
    let port = host_params.port.unwrap_or(22);

    Ok(SocketAddr::from_str(&format!("{}:{}", host_name, port))?)
}

fn get_first_identity_file(host_params: &HostParams) -> Result<PathBuf, Box<dyn Error>> {
    let identity_file = host_params
        .identity_file
        .clone()
        .expect("identity_file is required");
    let identity_file = identity_file
        .into_iter()
        .next()
        .expect("identity_file is required");

    Ok(identity_file)
}
