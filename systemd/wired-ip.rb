#!/usr/bin/env ruby
require 'erb'

class WiredIpService
  extend ERB::DefMethod
  def_erb_method :to_unit, File.expand_path('../wired-ip.service.erb', __FILE__)

  def initialize(opts)
    @opts = opts
  end

  def interface
    @opts[:interface] || detect_interface
  end

  def detect_interface
    Dir.entries('/sys/class/net').find do |fname|
      not %w[. .. lo].include? fname
    end
  end

  def gateway
    @opts[:gateway] || '192.168.0.1'
  end

  def netmask
    @opts[:netmask] || 24
  end

  def address
    @opts[:address] || '192.168.0.100'
  end
end

require 'optparse'

opts = {}
OptionParser.new.tap do |parser|
  parser.on('-g GATEWAY') { |v| opts[:gateway] = v }
  parser.on('-a ADDRESS') { |v| opts[:address] = v }
  parser.on('-n NETMASK') { |v| opts[:netmask] = v.to_i }
end.parse! ARGV

puts WiredIpService.new(opts).to_unit
