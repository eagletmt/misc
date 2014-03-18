#!/usr/bin/env ruby
require 'resolv'

begin
  gem 'rainbow'
  require 'rainbow'
rescue Gem::LoadError
  class Rainbow
    class NullPresenter
      def initialize(str)
        @str = str
      end

      def respond_to_missing?(symbol, include_private)
        true
      end

      def method_missing(*)
        @str
      end
    end

    def wrap(str)
      NullPresenter.new(str)
    end
  end
end

@rainbow = Rainbow.new
class << @rainbow
  def type(str)
    wrap(str).yellow
  end

  def addr(str)
    wrap(str).blue
  end

  def name(str)
    wrap(str).green
  end
end

def resolve_name(dns, name)
  if name =~ Resolv::IPv4::Regex
    return [name]
  end

  dns.each_resource(name, Resolv::DNS::Resource::IN::MX) do |mx|
    puts "#{name} #{@rainbow.type('MX')} #{@rainbow.name(mx.exchange.to_s)} #{mx.preference}"
  end

  begin
    loop do
      cname = dns.getresource(name, Resolv::DNS::Resource::IN::CNAME)
      puts "#{name} #{@rainbow.type('CNAME')} #{@rainbow.name(cname.name.to_s)}"
      name = cname.name.to_s
    end
  rescue Resolv::ResolvError
  end
  dns.getresources(name, Resolv::DNS::Resource::IN::A).map do |a|
    a.address.to_s.tap do |addr|
      puts "#{name} #{@rainbow.type('A')} #{@rainbow.addr(addr)}"
    end
  end
end

Resolv::DNS.open do |dns|
  ARGV.each do |arg|
    addrs = resolve_name(dns, arg)
    addrs.each do |addr|
      begin
        puts "#{addr} #{@rainbow.type('PTR')} #{@rainbow.name(dns.getname(addr))}"
      rescue Resolv::ResolvError
        puts "#{addr} #{@rainbow.type('PTR')} #{@rainbow.wrap('NONE').red}"
      end
    end
  end
end
