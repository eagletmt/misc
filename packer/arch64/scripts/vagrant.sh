#!/bin/sh
set -ex

mkdir -p -m700 ~/.ssh
curl -o ~/.ssh/authorized_keys https://raw.github.com/mitchellh/vagrant/master/keys/vagrant.pub
chmod 600 ~/.ssh/authorized_keys

sudo pacman --noconfirm -S virtualbox-guest-utils
sudo sh -c 'echo vboxsf > /etc/modules-load.d/vagrant-vbox.conf'

rm -f ~/VBoxGuestAdditions.iso
