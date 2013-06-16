liberator.plugins.pixiv = (function() {
  let libly = liberator.plugins.libly;
  let $LXs = libly.$U.getNodesFromXPath;
  let tags_cache = {};

  function retrieve_tt_from_illust(illust_id, cont) {//{{{
    let req = new libly.Request('http://www.pixiv.net/member_illust.php?mode=medium&illust_id=' + illust_id);
    req.addEventListener('onSuccess', function(res) {
      let tt = res.getHTMLDocument('//input[@name="tt"]');
      if (!tt) {
        liberator.echoerr('illust_id=' + illust_id + ' does not exist');
      } else {
        cont(tt[0].value);
      }
    });
    req.get();
  }//}}}

  function retrieve_tt_from_user(user_id, cont) {//{{{
    let req = new libly.Request('http://www.pixiv.net/member.php?id=' + user_id);
    req.addEventListener('onSuccess', function(res) {
      let tt = res.getHTMLDocument('//input[@name="tt"]');
      if (!tt) {
        liberator.echoerr('user_id=' + user_id + ' does not exist');
      } else {
        cont(tt[0].value);
      }
    });
    req.get();
  }//}}}

  let pixivManager = {
    bookmark_illust: function(id, tags, comment, next) {  // {{{
      let tt = content.document.querySelector('input[name="tt"]');
      if (!tt) {
        retrieve_tt_from_illust(id, cont);
      } else {
        cont(tt.value);
      }

      function cont(tt) {
        let params = {
          mode: 'add',
          tt: tt,
          id: id,
          type: 'illust',
          restrict: '0',
          tag: tags.map(function(t) encodeURIComponent(t)).join('+'),
          comment: comment,
        };
        let q = [k + '=' + params[k] for (k in params)].join('&');

        let req = new libly.Request('http://www.pixiv.net/bookmark_add.php', null, {postBody: q});
        req.addEventListener('onSuccess', next);
        req.post();
      }
    },  /// }}}
    bookmark_user: function(id, next) { // {{{
      let tt = content.document.querySelector('input[name="tt"]');
      if (!tt) {
        retrieve_tt_from_user(id, cont);
      } else {
        cont(tt.value);
      }

      function cont(tt) {
        let params = {
          mode: 'add',
          tt: tt,
          user_id: id,
          type: 'user',
          restrict: '0',
        };
        let q = [k + '=' + params[k] for (k in params)].join('&');
        let req = new libly.Request('http://www.pixiv.net/bookmark_add.php', null, {postBody: q});
        req.addEventListener('onSuccess', next);
        req.post();
      }
    },  /// }}}
    delete_bookmark_user: function(id, next) {  // {{{
      let tt = content.document.querySelector('input[name="tt"]');
      if (!tt) {
        retrieve_tt_from_user(id, cont);
      } else {
        cont(tt.value);
      }

      function cont(tt) {
        let params = {
          type: 'user',
          tt: tt,
          rest: 'show',
          'id%5B%5D': id,
          del: '%E3%80%80%E5%A4%96%E3%80%80%E3%81%99%E3%80%80',
        };
        let q = [k + '=' + params[k] for (k in params)].join('&');
        let req = new libly.Request('http://www.pixiv.net/bookmark_setting.php', null, {postBody: q});
        req.addEventListener('onSuccess', next);
        req.post();
      }
    },  // }}}
    get_entries: function(id, next) {  // {{{
      let url = 'http://www.pixiv.net/bookmark_detail.php?illust_id=' + id;
      let req = new libly.Request(url);
      req.addEventListener('onSuccess', function(res) {
        res.getHTMLDocument();
        let doc = res.doc;

        let obj = {};
        let span = doc.querySelector('.bookmark_detail_body > h3 > span');
        obj.count = span ? span.textContent.match(/^\d+/)[0] : '0';

        obj.entries = Array.map(doc.querySelectorAll('.bookmark_detail_body > ul'), function(ul) {
          let date = ul.querySelector('.days').textContent;
          let img = ul.querySelector('img');
          let tags = $LXs('descendant::a', ul).slice(3).map(function(a) a.textContent);
          return { date: date, imgsrc: img.src, user: img.alt, tags: tags };
        });

        next(obj);
      });
      req.get();
    },  // }}}
  };

  function unique(a)
    a.reduce(function(acc, x) {
      if (acc.indexOf(x) == -1) {
        acc.push(x);
      }
      return acc;
    }, []);

  commands.addUserCommand(['pixivBookmark'], 'pixiv bookmark',  // {{{
    function(args) {
      if (!buffer.URI.match(/www\.pixiv\.net\/member_illust\.php\?.*illust_id=(\d+)/)) {
        liberator.echoerr('not a pixiv illust page');
        return;
      }

      tags_cache = {};
      let id = RegExp.$1;
      pixivManager.bookmark_illust(id, args, '', function() liberator.echo('bookmarked'));
    },
    {
      completer: function(context, args) {
        let id = buffer.URI.match(/illust_id=(\d+)/)[1];
        let url = 'http://www.pixiv.net/bookmark_add.php?type=illust&illust_id=' + id;
        context.compare = void 0;
        if (tags_cache[url]) {
          context.title = ['tag (cached)'];
          context.completions = [[t, ''] for each(t in tags_cache[url]) if (args.indexOf(t) == -1)];
        } else {
          let req = new libly.Request('http://www.pixiv.net/bookmark_add.php?type=illust&illust_id=' + id);
          req.addEventListener('onSuccess', function(res) {
            let tags = unique(
              res.getHTMLDocument('//*[contains(concat(" ",normalize-space(@class)," "), " tag ")]')
              .map(function(e) e.getAttribute('data-tag')));
            let m = res.responseText.match(/pixiv\.context\.tags = ('.+');/);
            if (m) {
              let h = JSON.parse(eval(m[1]));
              for (tag in h) {
                if (h.hasOwnProperty(tag)) {
                  tags.push(tag);
                }
              }
              tags = unique(tags);
            } else {
              liberator.echoerr('pixiv_bookmark: failed to extract pixiv.context.tags');
            }

            tags_cache[url] = tags;
            context.title = ['tag'];
            context.completions = [[t, ''] for each(t in tags) if (args.indexOf(t) == -1)];
          });
          req.get();
        }
      },
      literal: -1,
    },
    true);  // }}}

  commands.addUserCommand(['pixivUserBookmark'], '[un]bookmark this user', // {{{
    function(args) {
      let id = content.window.wrappedJSObject.pixiv.context.userId;
      if (!id) {
        liberator.echoerr('cannot bookmark user here!');
        return;
      }
      if (args.bang) {
        pixivManager.delete_bookmark_user(id, function() liberator.echo('successfully unbookmarked'));
      } else {
        pixivManager.bookmark_user(id, function(res) {
          let m = res.responseText.match(/<a href="member\.php\?id=\d+">([^<]+)<\/a>([^<]+)/);
          liberator.echo(m[1] + m[2]);
        });
      }
    }, { bang: true, argCount: '0' }, true);  // }}}

  commands.addUserCommand(['pixivViewBookmark'], 'view pixiv bookmark', // {{{
    function() {
      if (!/http:\/\/www\.pixiv\.net\/member_illust\.php\?.*illust_id=(\d+)/.test(buffer.URI)) {
        liberator.echo('not pixiv illust page');
        return;
      }

      pixivManager.get_entries(RegExp.$1, function(r) {
        let dd = xml``;
        r.entries.forEach(function(e) {
          dd += xml`
            <dd class="liberator-pixiv-bookmark-entry" highlight="Completions" style="margin: 0; height: 18px;">
              <span class="liberator-pixiv-bookmark-date">${e.date}</span>
              <span class="liberator-pixiv-bookmark-icon"><img src=${e.imgsrc}/></span>
              <span class="liberator-pixiv-bookmark-user">${e.user}</span>
              <span class="liberator-pixiv-bookmark-tag" highlight="Tag" style="margin-left: 1em;">${e.tags.join(', ')}</span>
            </dd>`;
        });

        const TITLE = "\u3053\u306E\u30A4\u30E9\u30B9\u30C8\u3092\u30D6\u30C3\u30AF\u30DE\u30FC\u30AF\u3057\u3066\u3044\u308B\u30E6\u30FC\u30B6\u30FC";
        // XXX: This echoes raw string, not rendered one
        liberator.echo(xml`
          <dl id="liberator-pixiv-bookmark" style="margin: 0;">
            <dt highlight="CompTitle">${TITLE}  ${r.count}(${r.entries.length})</dt>
            ${dd}
          </dl>`);
      });
    }, { argCount: '0' }, true);  // }}}

  hints.addMode(  // hint mode for tombloo {{{
    'share-by-tombloo-pixiv',
    'Share by Tombloo (pixiv)',
    function(elem) {
        let tombloo = Cc['@brasil.to/tombloo-service;1'].getService().wrappedJSObject.Tombloo.Service;
        if (!tombloo) {
          liberator.echoerr('tombloo not found!');
          return;
        }

        let doc = content.document;
        let win = content.wrappedJSObject;
        let context = {
            document: doc,
            window:   win,
            title:    doc.title,
            target:   elem,
        };
        for (let p in win.location) {
            context[p] = win.location[p];
        }
        const name = 'Photo - Upload from Cache';
        let extractor = tombloo.check(context).filter(function(e) e.name == name);
        if (extractor.length == 0) {
          liberator.echoerr(name + ' is not available!');
          return;
        }
        extractor = extractor[0];
        tombloo.share(context, extractor, false);
    }, function() 'id("bigmode")/a/img | //table[starts-with(@id, "page")]/tbody/tr/td/a/img');  // }}}
  commands.addUserCommand(['pixivTombloo'], 'Share by Tombloo (pixiv)', function() hints.show('share-by-tombloo-pixiv'), { argCount: 0 });

  return pixivManager;
})();

// vim: set et sw=2 ts=2 sts=2 fdm=marker:

