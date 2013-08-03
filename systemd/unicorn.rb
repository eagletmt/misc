#!/usr/bin/env ruby
require 'erb'
require 'pathname'

class UnicornService
  extend ERB::DefMethod
  def_erb_method :to_unit, File.expand_path('../unicorn.service.erb', __FILE__)

  def initialize(opts)
    @opts = opts
  end

  def description
    @opts[:description] || 'Rails application'
  end

  def after
    @opts[:after] || %w[nginx.service postgresql.service]
  end

  def wants
    @opts[:wants] || after
  end

  def user
    @opts[:user] || ENV['USER']
  end

  def app_root
    Pathname.new(@opts[:app_root] || '.')
  end

  def gemfile_path
    app_root.join 'Gemfile'
  end

  def working_directory
    app_root
  end

  def config_path
    app_root.join 'config', 'unicorn.rb'
  end

  def pid_path
    app_root.join 'tmp', 'pids', 'unicorn.pid'
  end

  def restart_wait_seconds
    2
  end
end

require 'optparse'
opts = {}
OptionParser.new.tap do |parser|
  parser.on('-u USER') { |v| opts[:user] = v }
  parser.on('-r APP_ROOT') { |v| opts[:app_root] = v }
  parser.on('-a AFTER') { |v| opts[:after] ||= []; opts[:after] << v }
  parser.on('-w WANTS') { |v| opts[:wants] ||= []; opts[:wants] << v }
  parser.on('-d DESCRIPTION') { |v| opts[:description] = v }
end.parse! ARGV

puts UnicornService.new(opts).to_unit
