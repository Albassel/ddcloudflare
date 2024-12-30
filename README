A **fast** dynamic DNS client *written in Rust*. 

It finds your external ip address through https://cloudflare.com/cdn-cgi/trace and then uses the cloudflare token you provide to update your ip on cloudflare DNS servers dynamically.

# Configuration

This is configured using a .env file as follows:

```sh
TOKEN="<Your cloudflare token>"
ZONE="<Your cloudflare zone id>"
# A comma separated list of domains you want to update, each matching one of the provided records
DOMAINS="example.com, www.example.com"
# The interval in seconds between updating DNS entries
INTERVAL="70"
```

The .env file can be provided in the same directory as the binary or passed using command line arguments using the `-f path/to/env/file` option

# Contribution

If you think you can improve this project, feel free to issue a pull request and if the code is of good quality, it will be merged.
