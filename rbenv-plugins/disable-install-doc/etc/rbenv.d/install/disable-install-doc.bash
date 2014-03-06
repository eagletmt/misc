before_install disable_install_doc

disable_install_doc() {
  export RUBY_CONFIGURE_OPTS="--disable-install-doc $RUBY_CONFIGURE_OPTS"
}
