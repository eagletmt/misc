// ==UserScript==
// @name        Skip POSTD
// @namespace   https://wanko.cc
// @include     http://postd.cc/*
// @version     1
// @grant       none
// ==/UserScript==

jQuery(document).ready(function() {
  var href = document.querySelector('.block-text-original .ext-link').href;
  if (href) {
    window.location = href;
  }
});
