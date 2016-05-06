// ==UserScript==
// @name        Disable GitHub toolbar hotkey
// @namespace   https://wanko.cc
// @include     https://github.com/*
// @version     1
// @grant       none
// ==/UserScript==
(function() {
  var nodes = document.querySelectorAll('.js-toolbar-item[data-toolbar-hotkey]'), i;
  for (i = 0; i < nodes.length; i++) {
    nodes[i].removeAttribute('data-toolbar-hotkey');
  }
})();
