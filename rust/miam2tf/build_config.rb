MRuby::Build.new do |conf|
  conf.toolchain
  conf.bins = []
  conf.enable_debug

  conf.gembox 'stdlib'
  conf.gem core: 'mruby-compiler'
  conf.gem core: 'mruby-struct'
  conf.gem '.'
end
