#!/usr/bin/ruby
require 'time'
require 'json'
require 'twitter'
require 'pg'

client = Twitter::REST::Client.new do |c|
  c.consumer_key = ENV['TWITTER_CONSUMER_KEY']
  c.consumer_secret = ENV['TWITTER_CONSUMER_SECRET']
  c.access_token = ENV['TWITTER_ACCESS_TOKEN']
  c.access_token_secret = ENV['TWITTER_ACCESS_TOKEN_SECRET']
end


conn = PG::Connection.new(
  host: ENV['TWITTER_DB_HOST'],
  port: ENV['TWITTER_DB_PORT'],
  user: ENV['TWITTER_DB_USER'],
  password: ENV['TWITTER_DB_PASSWORD'],
  dbname: ENV['TWITTER_DB_NAME'],
)

ex = []
[:follower_ids, :friend_ids].each do |meth|
  cursor = -1
  while cursor != 0 do
    res = client.send(meth, cursor: cursor)
    res.attrs[:ids].each_slice(100) do |ids|
      begin
        client.users(*ids).each do |u|
          id = u.id
          name = u.screen_name
          t = conn.exec('SELECT name FROM users WHERE id = $1::bigint', [id])
          if t.none? { |x| x['name'] == name }
            puts "#{id}: #{t.map { |x| x['name'] }.inspect} -> #{name}"
            conn.exec("INSERT INTO users (id, name) VALUES ($1::bigint, $2::text)", [id, name])
          end
        end
      rescue Twitter::Error => e
        ex.push e
        if ex.size >= 10
          msgs = ["retried over 10 times!"] + ex.map(&:inspect)
          raise msgs.join("\n")
        end
        sleep 2
        retry
      end
    end
    cursor = res.attrs[:next_cursor]
  end
end

conn.close
