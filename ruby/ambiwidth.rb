#!/usr/bin/env ruby
# Usage
# % ./ambiwidth.rb
# % sudo cp UTF-8-CJK.gz /usr/share/i18n/charmaps/
# % sudo vim /etc/locale.gen  # Add "ja_JP.UTF-8 UTF-8-CJK"
# % sudo locale-gen

require 'open-uri'
require 'zlib'

utf8_charmap = '/usr/share/i18n/charmaps/UTF-8.gz'
east_asian_width_url = 'http://www.unicode.org/Public/UNIDATA/EastAsianWidth.txt'

class CharRange
  attr_accessor :left, :right

  def initialize(v = nil)
    if v
      @left = v
      @right = v+1
    else
      @left = @right = 0
    end
  end

  def to_s
    case
    when @left == @right
      ''
    when @left+1 == @right
      "#{format_codepoint @left}\t\t\t2\n"
    else
      "#{format_codepoint @left}...#{format_codepoint @right-1}\t2\n"
    end
  end

  def format_codepoint(c)
    if c < 0x10000
      sprintf '<U%04X>', c
    else
      sprintf '<U%08X>', c
    end
  end
end

ranges = []
cr = CharRange.new

open(east_asian_width_url).each_line do |line|
  if m = line.match(/\A([0-9A-F]+);A/)
    char = m[1].to_i 16
    if cr.right == char
      cr.right += 1
    else
      ranges << cr
      cr = CharRange.new char
    end
  end
end
ranges << cr

Zlib::GzipWriter.open('UTF-8-CJK.gz') do |gz|
  Zlib::GzipReader.open(utf8_charmap).each_line do |input|
    input.chomp!
    input.each_line do |line|
      if line == 'END WIDTH'
        gz.puts ranges
      end
      gz.puts line
    end
  end
end
