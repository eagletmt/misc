#!/usr/bin/ruby

`ps x`.each_line.map do |line|
  if line =~ %r!/libflashplayer\.so !
    line.match(/\b\d+\b/)[0]
  end
end.compact.each do |pid|
  `lsof -n -p #{pid}`.each_line.map do |line|
    line.chomp.split(/\s+/)
  end.select do |s|
    s[4] == 'REG' and s[9] and s[9] == '(deleted)' and s[8].start_with? '/tmp'
  end.each do |s|
    fd = s[3].match(/\b\d+/)[0]
    puts "/proc/#{pid}/fd/#{fd}"
  end
end
