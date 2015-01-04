// ==UserScript==
// @name        pixiv normal image link
// @namespace   https://wanko.cc/
// @include     http://www.pixiv.net/member_illust.php?mode=medium&illust_id=*
// @version     1
// @grant       none
// ==/UserScript==

jQuery(function($){
  var div = document.querySelector('.works_display > .ui-modal-trigger');
  var origImg = document.querySelector('img.original-image[data-src]');
  if (div && origImg) {
    var display = div.parentNode;
    var img = div.querySelector('img');
    if (img) {
      var a = document.createElement('a');
      a.href = origImg.getAttribute('data-src');
      a.appendChild(img);
      display.replaceChild(a, div);
    }
  }
});
