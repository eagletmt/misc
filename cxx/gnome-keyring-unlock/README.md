# gnome-keyring-unlock

## Dependencies
- libgnome-keyring

## Install
```sh
mkdir build
cd build
cmake .. -DCMAKE_BUILD_TYPE=release -DCMAKE_INSTALL_PREFIX=/usr
make install
```

## Caveats
All libgnome-keyring functions used in this tiny program are deprecated.
