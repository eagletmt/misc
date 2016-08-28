#!/usr/bin/env ruby
require 'openssl'
require 'socket'

now = Time.now
# 30 days
threshold = 30 * 24 * 60 * 60

rc = 0
ARGV.each do |host|
  TCPSocket.open(host, 443) do |socket|
    context = OpenSSL::SSL::SSLContext.new
    context.verify_mode = OpenSSL::SSL::VERIFY_PEER
    context.set_params
    ssl_socket = OpenSSL::SSL::SSLSocket.new(socket, context)
    ssl_socket.hostname = host
    ssl_socket.connect
    ssl_socket.post_connection_check(host)

    cert = ssl_socket.peer_cert
    if cert.not_after - now < threshold
      $stderr.puts "#{host}: Expires at #{cert.not_after}"
      rc += 1
    end
  end
end
exit rc
