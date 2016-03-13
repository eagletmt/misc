# fc-find
Find fonts which have the given characters.

## Build
```bash
cmake . -DCMAKE_BUILD_TYPE=release -DCMAKE_INSTALL_PREFIX=/usr
make install
```

## Usage
```
% fc-find 🍣
Noto Emoji
Symbola
Noto Color Emoji

% fc-find U+1F363
Noto Emoji
Symbola
Noto Color Emoji
```
