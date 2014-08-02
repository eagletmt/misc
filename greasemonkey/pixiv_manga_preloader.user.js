// ==UserScript==
// @name        pixiv manga preloader
// @namespace   http://wanko.cc/
// @include     http://www.pixiv.net/member_illust.php?mode=manga&illust_id=*
// @version     1
// @grant       none
// ==/UserScript==
jQuery(function ($) {
  $('img[data-src]').each(function (index, img) {
    img.src = img.dataset.src;
  });
});
