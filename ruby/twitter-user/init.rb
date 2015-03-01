#!/usr/bin/ruby
require 'pg'

conn = PG::Connection.new(
  host: ENV['TWITTER_DB_HOST'],
  port: ENV['TWITTER_DB_PORT'],
  user: ENV['TWITTER_DB_USER'],
  password: ENV['TWITTER_DB_PASSWORD'],
  dbname: ENV['TWITTER_DB_NAME'],
)
conn.exec "DROP TABLE IF EXISTS users"
conn.exec "CREATE TABLE users (id bigint NOT NULL, name text NOT NULL, updated_at timestamp DEFAULT now())"
conn.close
