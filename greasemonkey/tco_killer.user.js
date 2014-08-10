// ==UserScript==
// @name           t.co killer
// @namespace      http://wanko.cc/
// @include        https://twitter.com/*
// @grant          none
// @version        1
// ==/UserScript==

(function() {
  window.addEventListener('load', function() {
    var observer = new MutationObserver(function(mutations) {
      mutations.forEach(function(mutation) {
        onInserted(mutation.target);
      });
    });
    observer.observe(document, { childList: true, subtree: true });
  }, false);

  function onInserted(elt) {
    if (elt instanceof HTMLElement) {
      Array.prototype.forEach.call(elt.querySelectorAll('.twitter-timeline-link'), killTCo);
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
