// ==UserScript==
// @name           AOJ show all rows
// @namespace      http://wanko.cc/
// @include        http://judge.u-aizu.ac.jp/onlinejudge/*
// @grant          GM_addStyle
// @version        1
// ==/UserScript==

setTimeout(function() {
  var $ = unsafeWindow.$;
  if ($.jgrid) {
    $('.ui-jqgrid-bdiv').css('height', 'auto');
    var list = $('#list5');
    var n = list.jqGrid('getGridParam', 'records');
    if (n) {
      list.jqGrid('setGridParam', {rowNum: n}).trigger('reloadGrid');
    }
  } else {
    var t = document.getElementById('tableRanking');
    if (t) {
      GM_addStyle('#page .wrapper { width: 800px !important; }');
      $('tr.dat').show();
      $('.date').hide();
      $('.wrapper > .pagenavi').hide();
      t.removeAttribute('width');
    }
  }
}, 100);
