# ntv-vod
Save NTV Ondemand videos.
http://vod.ntv.co.jp/top/

## Requirements
- [rtmpsuck](http://rtmpdump.mplayerhq.hu/)
- Xvfb
- Firefox
    - flashplugin
- sudo

## Usage
```sh
bundle exec ./ntv-vod.rb -c 5257 -u rtmpsuck -d /tmp
```
