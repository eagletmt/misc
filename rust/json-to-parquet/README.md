# json-to-parquet
Convert JSON Lines to Parquet using given schema.

## Usage
This example uses [pqrs](https://github.com/manojkarthick/pqrs) to see Parquet file content.

```
% cat schema.txt
message foo {
  required int64 t (timestamp(millis, true));
  optional int32 n;
  optional binary s (string);
  optional int64 t2 (timestamp(millis, true));
  optional double d;
}
% cat input.jsonl
{"t":"2022-09-03T10:14:42.831Z","n":1,"s":"2","t2":"2022-09-05T11:15:43.942Z"}
{"t":"2022-09-04T06:13:22.033Z"}
{"t":"2022-09-04T06:13:22.033Z","n":null,"s":null}
% json-to-parquet -s schema.txt -i input.jsonl -o output.parquet
% pqrs schema output.parquet
Metadata for file: output.parquet

version: 1
num of rows: 3
created by: parquet-rs version 21.0.0
message foo {
  REQUIRED INT64 t (TIMESTAMP(MILLIS,true));
  OPTIONAL INT32 n;
  OPTIONAL BYTE_ARRAY s (STRING);
  OPTIONAL INT64 t2 (TIMESTAMP(MILLIS,true));
  OPTIONAL DOUBLE d;
}
% pqrs cat --json output.parquet

####################
File: output.parquet
####################

{"t":"2022-09-03 10:14:42.831","n":1,"s":"2","t2":"2022-09-05 11:15:43.942"}
{"t":"2022-09-04 06:13:22.033"}
{"t":"2022-09-04 06:13:22.033"}
```
