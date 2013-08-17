(function() {
  Tombfix.Service.extractors.register({
    name: 'Photo - Twitter',
    ICON: Models.Twitter.ICON,

    getThumbnail: function(ctx) {
      var e = ctx.document.querySelector('.twitter-timeline-link.media-thumbnail[data-resolved-url-large]');
      return e ? e.getAttribute('data-resolved-url-large') : null;
    },

    check: function(ctx) {
      if (!ctx.href.match('//twitter.com/.*?/(status|statuses)/\\d+')) {
        return false;
      }
      return this.getThumbnail(ctx) !== null;
    },

    extract: function(ctx) {
      var title = ctx.title.substring(0, ctx.title.indexOf(': '));
      var text = ctx.document.querySelector('.js-tweet-text').textContent;
      return {
        type: 'photo',
        item: title + ': ' + text,
        itemUrl: this.getThumbnail(ctx),
      };
    },
  });
})();
