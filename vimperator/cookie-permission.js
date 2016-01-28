(function() {
  let manager = Cc['@mozilla.org/permissionmanager;1'].getService(Ci.nsIPermissionManager);

  function permissionGenerator() {
    let enumerator = manager.enumerator;
    while (enumerator.hasMoreElements()) {
      let permission = enumerator.getNext().QueryInterface(Ci.nsIPermission);
      if (permission.type === 'cookie') {
        yield permission;
      }
    }
  }

  function addPermission(origin, perm) {
    manager.add(makeURI(origin), 'cookie', perm);
  }

  function removePermission(origin) {
    manager.remove(origin, 'cookie');
  }

  function hostCompleter(context, args) {
    context.title = ['Origin'];
    context.completions = [[content.window.location.origin]];
  }

  let PERM_DESC = {};
  PERM_DESC[manager.UNKNOWN_ACTION] = 'Unknown';
  PERM_DESC[manager.ALLOW_ACTION] = 'Allow';
  PERM_DESC[manager.DENY_ACTION] = 'Deny';
  PERM_DESC[manager.PROMPT_ACTION] = 'Prompt';
  PERM_DESC[8] = 'Allow for Session';

  function permissionCompleter(context, args) {
    context.title = ['Origin', 'Permission'];
    // FIXME: p.host is always undefined.
    context.completions = [for (p of permissionGenerator()) if (p.host) [p.host, PERM_DESC[p.capability]]];
  }

  // XXX: For Firefox 44+
  // https://blog.mozilla.org/addons/2015/10/14/breaking-changes-let-const-firefox-nightly-44/
  const Command = commands.getUserCommands()[0].constructor;
  commands.addUserCommand(['cp', 'cookiePermission'], 'Cookie Permission',
    function() {},
    {
      subCommands: [
        new Command(['allow'], 'Allow', function(args) addPermission(args.literalArg, manager.ALLOW_ACTION), { completer: hostCompleter, literal: 0}),
        new Command(['deny'], 'Deny', function(args) addPermission(args.literalArg, manager.DENY_ACTION), { completer: hostCompleter, literal: 0}),
        new Command(['session'], 'Allow for Session', function(args) addPermission(args.literalArg, 8), { completer: hostCompleter, literal: 0}),
        new Command(['remove'], 'Remove', function(args) removePermission(args.literalArg), { completer: permissionCompleter, literal: 0 }),
      ],
    }, true);
})();
