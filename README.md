# multiexec

Continuously execute a command on multiple hosts simultaneously.

## Install

```console
$ cargo install --git https://github.com/0x6b/multiexec
```

## Uninstall

```console
$ rm $(which multiexec)
```

## Usage

```
A tool to execute commands on multiple servers

Usage: multiexec [OPTIONS] <COMMAND>

Arguments:
  <COMMAND>  Command to execute

Options:
  -s, --ssh-config-path <SSH_CONFIG_PATH>
          Path to ssh config file. Defaults to "~/.ssh/config"
  -i, --interval <INTERVAL>
          Interval in seconds to execute the command. Defaults to 10 [default: 10]
  -n, --nodes <NODES>
          Comma separated list of nodes to execute the command on [default: node1,node2,node3,node4]
  -h, --help
          Print help
  -V, --version
          Print version
```

`multiexec` uses your SSH config to determine hostname, identity file, port number (default `22`), and user (default `root`).

```
Host node1
    HostName     192.168.0.10
    IdentityFile ~/.ssh/id_rsa
    User         root
    Port         22
```

Then run:

```shell
$ multiexec --node node1,node2,node3 "uname -a"
```
