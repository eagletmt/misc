Gem::Specification.new do |spec|
  spec.name = 'gmail-checker'
  spec.summary = 'check gmail'
  spec.homepage = 'https://github.com/eagletmt/misc/ruby/gmail-checker'
  spec.author = 'eagletmt'
  spec.email = 'eagletmt@gmail.com'
  spec.version = '0.1'

  spec.add_dependency 'nokogiri'
  spec.add_dependency 'pit'
  spec.add_dependency 'term-ansicolor'

  spec.executable = 'gmail.rb'
  spec.files = ['bin/gmail.rb']
end

