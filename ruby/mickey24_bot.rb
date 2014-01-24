#!/usr/bin/ruby
# coding: utf-8

class Brainfuck
  attr_accessor :mem, :ptr, :code, :inits, :output

  def initialize
    @inits = []
    @mem = []
    @ptr = 0
    @code = ''
    @output = []
  end

  def add_tail(c)
    r = Marshal.load(Marshal.dump(self))
    r.ptr = @mem.each_with_index.min_by do |m, i|
      [(m-c+256) % 256, (c-m+256) % 256].min + (i-@ptr).abs
    end[1]

    if r.ptr < @ptr
      r.code += '<' * (@ptr-r.ptr)
    elsif r.ptr > @ptr
      r.code += '>' * (r.ptr-@ptr)
    end

    if (r.mem[r.ptr]-c+256)%256 < (c-r.mem[r.ptr]+256)%256
      r.code += '-' * ((r.mem[r.ptr]-c+256)%256)
    elsif (r.mem[r.ptr]-c+256)%256 > (c-r.mem[r.ptr]+256)%256
      r.code += '+' * (c - r.mem[r.ptr])
    end
    r.code += '.'
    r.mem[r.ptr] = c
    r.output += [c]
    r
  end

  def add_head(c)
    r = Brainfuck.new
    n = 256.step(0, -6).min_by do |i|
      j = (256-i) / 6
      v = j/4
      if j%4 != 0
        if j%4 == 3
          v += 2 + 1 + 1
        else
          v += 2 + j%4
        end
      end
      (i-c).abs + v
    end
    r.inits = @inits + [n]
    r.rebuild_prelude
    @output.each do |d|
      r = r.add_tail(d)
    end
    r.add_tail(c)
  end

  def rebuild_prelude
    @mem = Marshal.load(Marshal.dump(@inits))
    @code = '++++++[>++++['
    six = []
    @inits.each do |m|
      t = 256-m
      @code += '>' + '-' * (t/24)
      six += [(t % 24)/6]
    end
    while six.last == 0
      six.pop
    end
    @code += '<' * @inits.size
    @code += '-]'
    six.each do |i|
      @code += '>' + '-'*i
    end
    @code += '<' * six.size
    @code += '<-]>>'
    self
  end
end

def mickey24_bot(s, limit = 140)
  target = s.unpack('C*')
  pq = [Brainfuck.new.add_head(target.shift)]
  target.each do |c|
    pq = pq.map do |bf|
      [bf.add_head(c), bf.add_tail(c)].select do |b|
        b.code.size + '@mickey24_bot '.size <= limit
      end
    end.flatten.sort_by do |b|
      b.code.size
    end
  end
  pq[0]
end

r = mickey24_bot(ARGF.read.chomp)
if r
  s = "@mickey24_bot #{r.code}"
  puts s.size
  puts s
else
  puts 'fail!'
end
