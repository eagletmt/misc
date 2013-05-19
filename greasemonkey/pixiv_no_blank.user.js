// ==UserScript==
// @name        pixiv no blank
// @namespace   http://wanko.cc/
// @include     http://www.pixiv.net/member_illust.php?mode=medium*
// @grant       none
// @version     1
// ==/UserScript==

for (let [, a] in Iterator(document.querySelectorAll('.works_display a'))) {
  a.removeAttribute('target');
}
