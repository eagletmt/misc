(function() {
  function getCookieString(uri)
    Cc['@mozilla.org/cookieService;1'].getService(Ci.nsICookieService).
      getCookieString(makeURI(uri), null);

  function show(x) liberator.echo(x);
  function copy(x) util.copyToClipboard(x);

  [['show', show], ['copy', copy]].forEach(function([mode, f]) {
    hints.addMode(
      'cookie-' + mode,
      mode + ' cookie for the selected link/image',
      function(elem) {
        let link = elem.href || elem.src;
        if (link) {
          f(getCookieString(link));
        } else {
          liberator.echoerr('cannot determine the link');
        }
      }, function() '//img | //a'
    );
  });


  commands.addUserCommand(['cookie'], 'Show cookie for the current page', function() show(getCookieString(buffer.URI)), { argCount: 0 });
  commands.addUserCommand(['cookieCopy'], 'Copy cookie for the current page', function() copy(getCookieString(buffer.URI)), { argCount: 0 });
  commands.addUserCommand(['cookieHint'], 'Show cookie for the selected link/image', function() hints.show('cookie-show'), { argCount: 0 });
  commands.addUserCommand(['cookieHintCopy'], 'Copy cookie for the selected link/image', function() hints.show('cookie-copy'), { argCount: 0 });

  liberator.plugins.cookie = {
    getCookieString: getCookieString,
  };
})();
