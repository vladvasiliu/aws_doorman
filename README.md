This is an utility that attempts to facilitate SSH access to AWS instances by updating a security group.

The main use case is when a direct connection is necessary, e.g when using [Mosh], which requires UDP.

In the general case, prefer connecting via [AWS Session Manager].


## Status

This project is in a very early stage. It does nothing useful as of yet.


## What it does

1. Get instance details:
    * External IP
    * Attached security groups
    * State
    
2. Get security group rules
    1. If rule with same GUID exists, update it
    2. If rule with GUID doesn't exist, creates it

3. Start external program


## Build

This is developed using the latest stable Rust. Development happens on Linux and MacOS but it may work on Windows too.

To build:

```sh
cargo build --release
```

### Requirements

These are the requirements for building. They are needed by [external-ip] which depends on c-ares.

* automake
* libssl-dev
* libtool


### Notable dependencies

* [external-ip]: Library to guess external IP from various sources
* [rusoto]: Rust AWS SDK


## Development

Useful plugins:

* [cargo-with]: Allows running a wrapper program to set environment variables.
Useful for temporary AWS credentials during development.


## License


This project is released under the terms of the GNU General Public License, version 3.
Please see [`COPYING`](COPYING) for the full text of the license.


[aws session manager]: https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-getting-started-enable-ssh-connections.html "AWS Session Manager Plugin"
[cargo-with]: https://lib.rs/crates/cargo-with "cargo-with"
[mosh]: https://mosh.org/ "Mosh"
[external-ip]: https://docs.rs/external-ip/ "external-ip"
[rusoto]: https://rusoto.org/ "Rusoto"
