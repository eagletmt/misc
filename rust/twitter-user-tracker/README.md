# twitter-user-tracker
Track screen_name of the specified user's followers and friends.

## Setup
### Twitter
Get consumer key and consumer secret for [Application-only authentication](https://dev.twitter.com/oauth/application-only).

### PostgreSQL
Initialize tables with init.sql.

```sh
psql -f init.sql
```

## Usage
```sh
export CONSUMER_KEY=xxx CONSUMER_SECRET=yyy POSTGRES_URL=postgres://eagletmt@localhost
twitter-user-tracker 25799899
```
