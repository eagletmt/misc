# ltsv-select
Filter LTSV records.

## Usage

```
% ltsv-select -l time -l upstream_response_time -l request_uri -l request_method /var/log/nginx/access.log
% ltsv-select -l time -l upstream_response_time -l request_uri -l request_method < /var/log/nginx/access.log
```

## Build

```sh
cargo build --release
```
