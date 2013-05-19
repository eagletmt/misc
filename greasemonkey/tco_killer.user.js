// ==UserScript==
// @name           t.co killer
// @namespace      http://wanko.cc/
// @include        https://twitter.com/*
// @grant          none
// @version        1
// ==/UserScript==

(function() {
  window.addEventListener('load', function() {
    document.addEventListener('DOMNodeInserted', onInserted, false);
    document.addEventListener('DOMAttrModified', onModified, false);
    Array.prototype.forEach.call(document.querySelectorAll('.twitter-timeline-link'), killTCo);
  }, false);

  function onInserted(evt) {
    var elt = evt.target;
    if (elt instanceof HTMLElement) {
      Array.prototype.forEach.call(elt.querySelectorAll('.twitter-timeline-link'), killTCo);
    }
  }

  function onModified(evt) {
    var elt = evt.target;
    if (elt.getAttribute('class') == 'twitter-timeline-link') {
      killTCo(elt);
    }
  }

  function killTCo(a) {
    var u = a.dataset.ultimateUrl || a.dataset.expandedUrl;
    if (u && u != a.href) {
      a.href = u;
      a.textContent = u;
    }
  }
})();
