mod node;

use std::error::Error;
use std::{
    fs::File,
    io::{BufReader, Read},
    net::{SocketAddr, TcpStream},
    str::FromStr,
    time::Duration,
};

use crate::node::Node;
use dirs::home_dir;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use ssh2::Session;
use ssh2_config::{HostParams, SshConfig};
use structopt::StructOpt;

static TICK_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";

#[derive(Debug, StructOpt)]
struct Args {
    /// Command to execute.
    command: String,

    /// Path to ssh config file. Defaults to "~/.ssh/config".
    #[structopt(short, long)]
    ssh_config_path: Option<String>,

    /// Comma separated list of nodes to execute the command on. Node can be specified by "node1" or "1".
    #[structopt(
        short,
        long,
        value_delimiter = ",",
        default_value = "1,2,3,4"
    )]
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

        tasks.push(tokio::spawn(exec(args.command.clone(), ssh_config.query(node), pb)));
    });

    for task in tasks {
        task.await.unwrap();
    }

    Ok(())
}

async fn exec(command: String, host_params: HostParams, pb: ProgressBar) {
    let HostParams {
        host_name,
        port,
        user,
        identity_file,
        ..
    } = host_params;

    let host_name = host_name.as_ref().expect("hostname is required");
    let port = port.unwrap_or(22);
    let user = user.clone().unwrap_or("root".to_string());

    let identity_file = identity_file.as_ref().expect("identity_file is required");
    let identity_file = identity_file.first().expect("identity_file is required");

    let mut interval = tokio::time::interval(Duration::from_millis(10000));
    for _ in 0.. {
        interval.tick().await;

        // Connect to the remote server
        let stream = TcpStream::connect_timeout(
            &SocketAddr::from_str(&format!("{}:{}", &host_name, &port)).unwrap(),
            Duration::from_secs(10),
        )
        .unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .unwrap();

        let mut sess = Session::new().unwrap();
        sess.set_tcp_stream(stream);
        sess.set_timeout(10 * 1000);
        sess.handshake().unwrap();

        // Authenticate with the remote server
        sess.userauth_pubkey_file(&user, None, identity_file, None)
            .unwrap();
        assert!(sess.authenticated());

        // Execute a ≠command on the remote server
        let mut channel = sess.channel_session().unwrap();
        channel.exec(&command).unwrap();

        // get current date and time in RFC3399 format
        let now = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        // Read the output of the command
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        let result = s
            .lines()
            .map(|line| format!("{now} - {}", line))
            .collect::<Vec<String>>()
            .join("\n");

        pb.set_message(result);
        pb.inc(1);
        // Close the channel and the session
        channel.send_eof().unwrap();
        channel.wait_close().unwrap();
        sess.disconnect(None, "disconnect", None).unwrap();
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
