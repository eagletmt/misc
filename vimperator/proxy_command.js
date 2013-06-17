/*
 * Commandline interface for proxy settings.
 * This requires revision >= 4430 for subCommands.
 *
 * examples:
 *  - display current proxy type
 *    :proxy
 *  - do not use proxy
 *    :proxy direct
 *  - manual proxy configuration
 *    :proxy manual -http proxy.example.com:3128 -no-proxy localhost
 *  - proxy auto-configuration (PAC)
 *    :proxy pac -reload file:///home/eagletmt/proxy.pac
 *  - auto-detect proxy settings
 *    :proxy auto
 *  - use system proxy settings
 *    :proxy system
 */

(function() {
  function make_completer(type)/*{{{*/
    function(context, args) {
      context.title = [type + ' proxy', 'URL'];
      let host = options.getPref('network.proxy.' + type);
      if (host) {
        let port = options.getPref('network.proxy.' + type + '_port');
        return [[host + ':' + port, 'current ' + type + ' proxy URL']];
      } else {
        return [];
      }
    };/*}}}*/

  function no_proxy_completer(context, args) {//{{{
    let host = options.getPref('network.proxy.no_proxies_on');
    return host ? [[host, 'current no-proxy']] : [];
  }//}}}

  function set_proxy_for(type, arg) {//{{{
    let [host, port] = arg.split(/:/);
    options.setPref('network.proxy.' + type, host);
    options.setPref('network.proxy.' + type + '_port', parseInt(port));
  }//}}}

  const desc = [//{{{
    'no proxy',
    'manual proxy configuration',
    'proxy auto-configuration (PAC)',
    '',
    'auto-detect proxy settings',
    'use system proxy settings',
  ];//}}}

  commands.addUserCommand(['proxy'], 'proxy setting',
    function() {
      let type = options.getPref('network.proxy.type');
      liberator.echo('current proxy type = ' + type + ': ' + desc[type]);
    },
    {
      subCommands: [
        new Command(['direct'], desc[0], function() options.setPref('network.proxy.type', 0)),
        new Command(['manual'], desc[1],//{{{
          function(args) {
            options.setPref('network.proxy.type', 1);
            ['http', 'ssl', 'ftp', 'socks'].forEach(function(type) {
              if (args['-' + type]) {
                set_proxy_for(type, args['-' + type]);
              }
            });
            if (args['-no-proxy']) {
              options.setPref('network.proxy.no_proxies_on', args['-no-proxy']);
            }
          }, {
            options: [
              [['-http'], commands.OPTION_STRING, null, make_completer('http')],
              [['-ssl'], commands.OPTION_STRING, null, make_completer('ssl')],
              [['-ftp'], commands.OPTION_STRING, null, make_completer('ftp')],
              [['-socks'], commands.OPTION_STRING, null, make_completer('socks')],
              [['-no-proxy'], commands.OPTION_STRING, null, no_proxy_completer],
            ],
          }),//}}}
        new Command(['pac'], desc[2],//{{{
          function(args) {
            options.setPref('network.proxy.type', 2);
            if (args.literalArg) {
              options.setPref('network.proxy.autoconfig_url', args.literalArg);
            }
            if (args['-reload']) {
              Cc['@mozilla.org/network/protocol-proxy-service;1'].getService().reloadPAC();
            }
          }, {
            literal: 0,
            options: [
              [['-reload', '-r'], commands.OPTION_NOARG],
            ],
            completer: function(context, args) {
              context.title = ['PAC URL'];
              context.completions = [[options.getPref('network.proxy.autoconfig_url'), 'current PAC URL']];
            },
          }),//}}}
        new Command(['auto'], desc[4], function() options.setPref('network.proxy.type', 4)),
        new Command(['system'], desc[5], function() options.setPref('network.proxy.type', 5)),
      ],
    }, true);
})();
// vim: set sw=2 ts=2 sts=2 et fdm=marker:
