a simple http server like `python -m http.server` but:

* written in rust with actix, should be faster
* allow concurrency
* download whole directories in .tar format
* better auto index
* maybe announce itself on mDNS (avahi)
* maybe compress

```
USAGE:
    http-server [OPTIONS] [port]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --bind <ADDRESS>       Specify alternate bind address [default: 0.0.0.0]
        --chdir <DIRECTORY>    Specify directory to server [default: .]

ARGS:
    <port>    Specify alternate port [default: 8000]
```

## FAQ

* Q: why .tar and not .zip? A: you can't stream a zip file efficiently, it needs to seek and write to the beggining of a file.
