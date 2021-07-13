# AWS Doorman

AWS Doorman is a simple utility for facilitating access to AWS resources that have restricted access by IP.

It keeps the entries of a prefix list up to date with the actual external IP.

How it works:

* It retrieves the computer's external IP and adds it to an AWS Managed Prefix List.
* It checks regularly what the IP is and updates the Prefix List entries as needed.
* It removes the IP when shutting down.
* It works on Managed Prefix List entries that have a specific description.

This is a tool I have developed as I've been working from home on a connection without a fixed IP address.
The main use is avoiding a VPN connection which tends to not work too well on spotty connections.

The main use case is when a direct connection is necessary, e.g when using [Mosh], which requires UDP, or for services
which don't have proxied access available, such as RDS.

In the general case, prefer connecting via [AWS Session Manager].


## Status

Development is in its early stages. It works for me ®, but your mileage may vary.


### Known limitations

* No IPv6 support.
* No multi-home support.


## ** CAUTION **

Please make sure you understand the implications of using this! **Everyone using your public IP will have the same
network access as you!** This may range from only people sharing your connection all the way to many unknown people in
the case of CGNAT and mobile connections!

Please make sure that you only grant the absolute minimum access and that the resources implement some other kind of
defense, such as some strong form of authentication! As traffic goes directly over the internet, make sure you're using
encryption! For example, RDS does not use encryption by default.

If at all possible, prefer using alternative means of connection, such as [AWS Session Manager], your company's VPN,
etc.


## Running

For the full list of options:

```
aws_doorman -h
```

There is no support for using AWS IAM Roles directly. It expects to retrieve the credentials from the usual places.

I recommend using it with [AWS Vault]. AWS Vault handles assuming roles and dealing with MFA.

Example using default credentials from `~/.aws/credentials`:

```
aws_doorman --prefix-list-id pl-1234567890abcdef1 --description some-description --interval 120
```

Example using AWS role *some-role* from AWS Vault:

```
aws-vault exec some-role -- aws_doorman --prefix-list-id pl-1234567890abcdef1 --description some-description --interval 120
```


## Building

Doorman is developed on up-to-date Linux and MacOS using the latest Rust toolchain.
It may work on Windows too, but it's not tested.

To build for production:

```sh
cargo build --release
```


## Development

Contributions in the form of pull requests, issues, etc are welcome.

Useful plugins:

* [cargo-with] - Allows running a wrapper program to set environment variables.
Useful for temporary AWS credentials during development.


## License

This project is released under the terms of the GNU General Public License, version 3.
Please see [`COPYING`](COPYING) for the full text of the license.


[aws vault]: https://github.com/99designs/aws-vault "AWS Vault"
[aws session manager]: https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-getting-started-enable-ssh-connections.html "AWS Session Manager Plugin"
[cargo-with]: https://lib.rs/crates/cargo-with "cargo-with"
[mosh]: https://mosh.org/ "Mosh"
