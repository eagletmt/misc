(function() {
  commands.addUserCommand(['ogimage'], 'Open og:image content', function() {
    let meta = content.document.querySelector('meta[property="og:image"]');
    if (meta) {
      liberator.open(meta.content);
    } else {
      liberator.echoerr('No og:image property found');
    }
  }, { argCount: 0 });
})();
