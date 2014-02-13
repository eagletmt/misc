#!/usr/bin/env ruby

EXIT_CODES = {
  ok: 0,
  warning: 1,
  critical: 2,
  unknown: 3,
}

UNIT = ARGV[0]
if UNIT.nil?
  $stderr.puts "Usage: #{$0} <unit-name>"
  exit EXIT_CODES[:unknown]
end

def finish(code, message)
  puts "#{UNIT} #{code.to_s.upcase}: #{message}"
  exit EXIT_CODES[code]
end

props = {}
`systemctl show #{UNIT}`.each_line do |line|
  prop, val = line.chomp.split('=', 2)
  props[prop] = val
end

if props['LoadState'] != 'loaded'
  finish(:critical, "Not loaded - #{props['LoadState']}")
end

if props['ActiveState'] != 'active'
  finish(:critical, "Not active - #{props['ActiveState']}")
end

finish(:ok, "active #{props['SubState']}")
