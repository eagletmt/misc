(function() {
  function getPlayer(f) {
    let iframe = content.document.querySelector('.speakerdeck-iframe');
    if (iframe) {
      return f(iframe.contentWindow.wrappedJSObject.player);
    } else {
      liberator.echoerr('speakerdeck-iframe not found');
      return;
    }
  }

  function slideCompleter(context, args) {
    getPlayer(function(player) {
      context.title = ['Slides', ''];
      context.compare = void 0;
      let n = player.count();
      let ary = new Array(n);
      for (let i = 0; i < n; i++) {
        ary[i] = [i, ''];
      }
      context.completions = ary;
    });
  }

  commands.addUserCommand(['speakerdeck'], 'speakerdeck controller',
    function() {
    }, {
      subCommands: [
        new Command(['n[ext]'], 'Go next slide', function() getPlayer(function(player) player.nextSlide())),
        new Command(['p[rev]'], 'Go previous slide', function() getPlayer(function(player) player.previousSlide())),
        new Command(['go'], 'Go to slide', function(args) getPlayer(function(player) player.goToSlide(parseInt(args[0], 10))), { argCount: 1, completer: slideCompleter }),
      ],
    }, true);
})();
