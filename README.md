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
USAGE:
    multiexec [OPTIONS] <command>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --interval <interval>                  Interval in seconds to execute the command. Defaults to 10 [default: 10]
    -n, --nodes <nodes>...                     Comma separated list of nodes to execute the command on [default:
                                               node1,node2,node3,node4]
    -s, --ssh-config-path <ssh-config-path>    Path to ssh config file. Defaults to "~/.ssh/config"

ARGS:
    <command>    Command to execute
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
