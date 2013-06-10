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

  commands.addUserCommand(['speakerdeck'], 'speakerdeck controller',
    function() {
    }, {
      subCommands: [
        new Command(['n[ext]'], 'Go next slide', function() getPlayer(function(player) player.nextSlide())),
        new Command(['p[rev]'], 'Go previous slide', function() getPlayer(function(player) player.previousSlide())),
      ],
    }, true);
})();
