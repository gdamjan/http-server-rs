[![Build Status](https://github.com/gdamjan/http-server-rs/workflows/build%20test%20lint/badge.svg)](https://github.com/gdamjan/http-server-rs/actions/)

A simple http server like `python -m http.server` but:

* written in rust with actix, should be faster
* allow concurrency
* download whole directories in .tar format
* fancier directory listing
* maybe announce itself on mDNS (avahi)

```
USAGE:
    http-server [OPTIONS] [PORT]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --bind <ADDRESS>       Specify alternate bind address [default: 0.0.0.0]
        --chdir <DIRECTORY>    Specify directory to server [default: .]

ARGS:
    <PORT>    Specify alternate port [default: 8000]
```

## FAQ

* Q: why .tar and not .zip? A: ~you can't stream a zip file efficiently, it needs to seek and write to the beggining of a file.~ will see.


## Release builds
```
cargo build --release
strip --strip-unneeded ./target/release/http-server
```

## See also:

* https://github.com/svenstaro/miniserve
