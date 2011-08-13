# flash.rb
For Linux only.

This displays the cache location of the Flash viewing via web browser such as Firefox and Google Chrome.

## Example Usage

    % flash.rb
    /proc/7155/fd/21

To retrieve the cache, simply `cp` it.

    % cp `flash.rb` video.mp4
