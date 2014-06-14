#!/usr/bin/env ruby
ENV['BUNDLE_GEMFILE'] ||= File.expand_path('Gemfile', File.dirname(__FILE__))
require 'bundler/setup'
Bundler.setup
require 'google_drive'
require 'pathname'

if ARGV.size < 2
  $stderr.puts "Usage: #{$0} /path/to/file title"
  exit 1
end

session = GoogleDrive.login('eagletmt@gmail.com', 'zjmcmdgdmkfdkijw')

segments = Pathname.new(ARGV[1]).to_enum(:descend).map { |v| v.basename.to_s }
title = segments.pop
collection = session.root_collection
segments.each do |segment|
  sub = collection.subcollection_by_title(segment)
  if sub.nil?
    sub = collection.create_subcollection(segment)
  end
  collection = sub
end
file = session.upload_from_file(ARGV[0], ARGV[1], convert: false)
puts "Uploaded to #{file.human_url}"
collection.add(file)
file.rename(title)
puts "Moved to #{file.human_url}"
