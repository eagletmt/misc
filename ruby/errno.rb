#!/usr/bin/ruby

ERRNOS = Errno.constants.map do |n|
  Errno.const_get(n).new
end.freeze

def find_errnos(arg)
  if arg =~ /\A\d+\z/
    [ERRNOS[arg.to_i]]
  else
    [Errno.const_get(arg.upcase.to_sym).new]
  end
rescue NameError
  ERRNOS.select { |e| e.message.include?(arg) }
end

ARGV.each do |arg|
  find_errnos(arg).each do |e|
    puts "#{e.errno}: #{e.class.name.sub(/^Errno::/, '')}: #{e.message}"
  end
end
