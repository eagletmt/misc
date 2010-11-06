#!/usr/bin/ruby
require 'time'
require 'net/https'
require 'nokogiri'
require 'term/ansicolor'
require 'pit'

class String
  include Term::ANSIColor
end

if ARGV.first == 'help'
  puts ['unread', 'anywhere', 'starred', 'voicemail', 'sent', 'spam', 'trash']
  exit 0
end

config = Pit.get('www.google.com', :require => {
  'username' => 'your username',
  'password' => 'your password',
})

uri = URI.parse "https://mail.google.com/mail/feed/atom/#{ARGV.first || ''}"
https = Net::HTTP.new uri.host, uri.port
https.use_ssl = true
cert_store = OpenSSL::X509::Store.new
cert_store.set_default_paths
https.cert_store = cert_store
https.verify_mode = OpenSSL::SSL::VERIFY_PEER
doc = https.start do |w|
  req = Net::HTTP::Get.new uri.path
  req.basic_auth config['username'], config['password']
  res = w.request req
  if res.is_a? Net::HTTPSuccess
    Nokogiri res.body
  elsif res.is_a? Net::HTTPForbidden
    puts 'incorrect username or password'
    exit 1
  else
    puts "Error: #{res.message}"
    exit 2
  end
end

ns = { 'atom' => 'http://purl.org/atom/ns#' }
puts "#{doc.at('feed/fullcount').text} mails"
doc.xpath('//atom:entry', ns).each do |entry|
  title = entry.at('title').text
  summary = entry.at('summary').text
  message_id = entry.at('link')['href'].match(/message_id=([0-9a-f]+)/)[1]
  link = "https://mail.google.com/mail/#inbox/#{message_id}"
  name = entry.at('author/name').text
  addr = entry.at('author/email').text
  issued = entry.at('issued').text
  t = Time.parse(issued).localtime.to_s rescue issued

  puts title.underline
  puts "  #{name.magenta}<#{addr}> [#{t.green}]"
  puts "  #{summary.bold}"
  puts "  #{link.cyan}"
  puts ''
end

