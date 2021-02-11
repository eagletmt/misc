MRuby::Build.new do |conf|
  conf.toolchain
  conf.bins = []
  conf.enable_debug

  conf.gembox 'stdlib'
  conf.gem core: 'mruby-compiler'
  conf.gem core: 'mruby-struct'
  conf.gem github: 'iij/mruby-regexp-pcre'
  conf.gem github: 'k0kubun/mruby-hashie'
  conf.gem '.'
end
