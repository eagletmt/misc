(function() {
  let klass = Cc['@lastpass.com/lastpass;1'];
  if (klass) {
    commands.addUserCommand(['lplogin'], 'Open LastPass login dialog', function() {
      klass.getService().wrappedJSObject.lpGetCurrentWindow().openDialog('chrome://lastpass/content/login.xul', '_blank', 'resizable,chrome,toolbar,centerscreen,modal');
    }, { argCount: '0' }, true);
  }
})();
