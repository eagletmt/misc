# Web Console

## Setup
```sh
sqlite3 webconsole.sqlite3 < schema.sql
```

## Build
```sh
go get -d .
go build
```

## Run
```sh
./webconsole -bind :3000
```
