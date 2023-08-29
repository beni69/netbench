# netbench

`netbench` is a simple network benchmarking tool. It is designed to be simple to
use and to be able to run on a wide variety of platforms.

See `netbench --help` for full usage instructions.

### Server

```sh
netbench server [optional host and port -H 127.0.0.1 -p 1234]
```

example docker command:

```sh
docker run -dp 42069:42069 --rm --name netbench ghcr.io/beni69/netbench
```

### Client

```sh
netbench client <server address>[:port]
```
