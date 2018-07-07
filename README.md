a simple http server like `python -m http.server` but:

* written in rust with actix, should be faster
* allow concurrency
* download whole directories in .tar format
* better auto index
* maybe announce itself on mDNS (avahi)
* maybe compress

Usage [TODO]:

```
http-server [--bind ADDRESS] [--chdir DIRECTORY] [port]

  port                  Specify alternate port [default: 8000]
  --bind ADDRESS        Specify alternate bind address [default: all interfaces]
  --chdir DIRECTORY     Specify directory to server [default: current directory]
```

## FAQ

* Q: why .tar and not .zip? A: you can't stream a zip file efficiently, it needs to write back in a file.
