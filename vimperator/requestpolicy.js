(function() {
  if (!window.requestpolicy) {
    liberator.echoerr('RequestPolicy is not installed');
    return;
  }

  function blockedDestinations()
    window.requestpolicy.menu._blockedDestinationsItems.map(function(item) item.getAttribute('label').trim());

  function allowedDestinations()
    window.requestpolicy.menu._allowedDestinationsItems.map(function(item) item.getAttribute('label').trim());

  function blockedCompleter(context, args) {
    window.requestpolicy.menu.prepareMenu();
    context.title = ['Host', 'Status'];
    context.completions = blockedDestinations().map(function(dest) [dest, 'Blocked']);
  }

  function allowedCompleter(context, args) {
    window.requestpolicy.menu.prepareMenu();
    context.title = ['Host', 'Status'];
    context.completions = allowedDestinations().map(function(dest) [dest, 'Allowed']);
  }

  function allowDestination(host) {
    window.requestpolicy.overlay.allowDestination(host);
  }

  function forbidDestination(host) {
    window.requestpolicy.overlay.forbidDestination(host);
  }

  commands.addUserCommand(['requestpolicy'], 'RequestPolicy', function() {},
  {
    subCommands: [
      new Command(['allow'], 'Allow', function(args) allowDestination(args.literalArg), { completer: blockedCompleter, literal: 0 }),
      new Command(['forbid'], 'Forbid', function(args) forbidDestination(args.literalArg), { completer: allowedCompleter, literal: 0 }),
      new Command(['log'], 'Toggle request log', function() window.requestpolicy.overlay.toggleRequestLog(), { argCount: 0 }),
    ],
  }, true);
})();
