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

  function addPermission(host, perm) {
    manager.add(makeURI('http://' + host), 'cookie', perm);
  }

  function removePermission(host) {
    manager.remove(host, 'cookie');
  }

  function hostCompleter(context, args) {
    context.title = ['Host'];
    context.completions = [[content.window.location.host]];
  }

  let PERM_DESC = {};
  PERM_DESC[manager.UNKNOWN_ACTION] = 'Unknown';
  PERM_DESC[manager.ALLOW_ACTION] = 'Allow';
  PERM_DESC[manager.DENY_ACTION] = 'Deny';
  PERM_DESC[manager.PROMPT_ACTION] = 'Prompt';
  PERM_DESC[8] = 'Allow for Session';

  function permissionCompleter(context, args) {
    context.title = ['Host', 'Permission'];
    context.completions = [[p.host, PERM_DESC[p.capability]] for (p in permissionGenerator())];
  }

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
