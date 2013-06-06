#!/usr/bin/ruby
require 'time'
require 'sqlite3'

user = ENV['USER']
SQLite3::Database.new File.expand_path("~/.Skype/#{user}/main.db") do |db|
  db.results_as_hash = true
  db.execute('SELECT chatname, author, timestamp, body_xml FROM Messages') do |row|
    next unless row['body_xml']
    t = Time.at row['timestamp']
    puts "[#{t}] #{row['author']}: #{row['body_xml']}"
  end
end
