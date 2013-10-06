#!/usr/bin/env ruby
require 'thread'
require 'timeout'
require 'open3'
require 'pathname'
require 'fileutils'
require 'optparse'
require 'pit'
require 'selenium-webdriver'
require 'headless'

class FileWatcher
  def initialize(path, opts = {})
    @path = path
    @interval = opts[:interval] || 1
    @count = opts[:count] || 15
  end

  def wait
    prev_size = -1
    n = 0
    loop do
      s = @path.size
      if prev_size == s
        n += 1
        if n == @count
          return
        end
      else
        puts "Current file size: #{s}"
        prev_size = s
        n = 1
      end
      sleep @interval
    end
  end
end

class OoyalaPlayer
  # NTV ondemand uses Ooyala player V2.
  # http://support.ooyala.com/developers/documentation/api/player_api_js.html

  def initialize(driver, player_id)
    @driver = driver
    @player_id = player_id
  end

  def state
    call :getState
  end

  def play
    call :playMovie
  end

  def title
    call :getTitle
  end

  def error_code
    call :getErrorCode
  end

  def error_text
    call :getErrorText
  end

  def call(sym)
    @driver.execute_script(<<"EOS")
var p = document.getElementById('#{@player_id}');
if (p === null) {
  return null;
}
if (typeof p.#{sym} !== 'function') {
  return null;
}
return p.#{sym}();
EOS
  end
end

class VodBrowser
  class PlayerError < StandardError
    attr_reader :code, :text
    def initialize(code, text)
      super("#{code}: #{text}")
      @code = code
      @text = text
    end
  end

  def initialize(driver)
    @driver = driver
    @player = OoyalaPlayer.new(@driver, 'ooplayer')
  end

  def login(email, password)
    @driver.navigate.to 'https://vod.ntv.co.jp/signin/'
    @driver.find_element(:id, 'login_uid').send_keys(email)
    @driver.find_element(:id, 'login_pwd').tap do |pwd|
      pwd.send_keys(password)
      pwd.submit
    end
    Selenium::WebDriver::Wait.new(timeout: 10, message: 'Login failed').until do
      @driver.current_url == 'http://vod.ntv.co.jp/top/'
    end
  end

  def visit_contents(contents_id)
    @driver.navigate.to "http://vod.ntv.co.jp/contentsViewer/?contentsId=#{contents_id}"
    self
  end

  def play
    loop do
      state = @player.state
      puts "State: #{state}"
      case state
      when nil
        # ignore
      when 'paused'
        @player.play
      when 'playing', 'buffering'
        return
      when 'error'
        raise PlayerError.new(@player.error_code, @player.error_text)
      else
        fail "Unknown state #{state}"
      end
      sleep 1
    end
  end

  def contents_title
    @player.title
  end
end

config = Pit.get('vod.ntv.co.jp', require: { 'email' => 'email', 'password' => 'password' })
# Note: save_dir must be writable to ENV['USER'] and rtmpsuck_user
opts = {
  rtmpsuck_user: 'rtmpsuck',
  save_dir: '/tmp',
  contents_id: nil,
}
OptionParser.new.tap do |parser|
  parser.on('-c', '--contents-id=ID') { |v| opts[:contents_id] = v.to_i }
  parser.on('-u', '--user=USER') { |v| opts[:rtmpsuck_user] = v }
  parser.on('-d', '--save-directory=DIR') { |v| opts[:save_dir] = v }
end.parse!(ARGV)

unless opts[:contents_id]
  $stderr.puts "No contents_id given"
  exit 1
end

rtmpsuck_user = 'rtmpsuck'
save_dir = '/tmp'

# Assumption: iptables -t nat -A OUTPUT -p tcp --dport 1935 -m owner \! --uid-owner #{opts[:rtmpsuck_user}} -j REDIRECT

system('sudo', '-k')
stdin, stdout, wait_thread = *Open3.popen2e('sudo', '-p', "[sudo] start rtmpsuck by '#{opts[:rtmpsuck_user]}' user: ", '-u', opts[:rtmpsuck_user], 'rtmpsuck', chdir: opts[:save_dir])
at_exit do
  unless stdin.closed?
    stdin.puts 'q'
    stdin.close
  end
end

streaming_notifier = Queue.new
saving_notifier = Queue.new

rtmpsuck_thread = Thread.new do
  path = nil
  stdout.each_line do |line|
    line.chomp!
    puts line
    case line
    when /\AStreaming on /
      streaming_notifier.push true
    when 'ERROR: Failed to start RTMP server, exiting!'
      streaming_notifier.push false
    when /\ASaving as: (.+)\z/
      path = Pathname.new(opts[:save_dir]).join($1)
      saving_notifier.push true
      Thread.new do
        FileWatcher.new(path).wait
        stdin.puts 'q'
        stdin.close
      end
    when /\AClosing connection/
      saving_notifier.push false
    end
  end
  path
end

unless streaming_notifier.pop
  $stderr.puts "rtmpsuck failed. Other process is using 1935?"
  exit 1
end

Headless.ly do
  driver = Selenium::WebDriver.for :firefox
  vod = VodBrowser.new(driver)
  vod.login(config['email'], config['password'])

  vod.visit_contents(opts[:contents_id]).play
  title = vod.contents_title
  puts "Title: #{title}"

  catch :success do
    loop do
      begin
        Timeout.timeout 30 do
          if saving_notifier.pop
            throw :success
          else
            puts 'Retrying...'
            driver.navigate.refresh
            vod.play
          end
        end
      rescue Timeout::Error
        $stderr.puts "Streaming doesn't start!"
        $stderr.puts "Is iptables configured properly?"
        exit 1
      end
    end
  end

  path = rtmpsuck_thread.value
  FileUtils.cp path.to_s, "#{title}.mp4"
  puts "Saved as #{title}.mp4"
  driver.quit
end
