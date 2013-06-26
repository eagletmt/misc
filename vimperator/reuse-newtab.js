(function() {
  let U = liberator.plugins.libly.$U;

  let tabopen = commands.get('tabopen');
  let open = commands.get('open');
  U.around(tabopen, 'action', function(next, args) {
    for each ([idx, tab] in tabs.browsers) {
      if (tab.currentURI.spec == 'about:newtab') {
        tabs.select(idx);
        return open.action.apply(open, args);
      }
    }
    return next();
  });
})();
