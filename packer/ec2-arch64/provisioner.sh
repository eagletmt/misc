#!/bin/sh
set -ex

cat <<'EOS' >> /etc/pacman.conf
[aur-eagletmt]
SigLevel = Required
Server = http://arch.wanko.cc/$repo/os/$arch
EOS
GPGKEY=C48DBD97
pacman-key --recv-keys $GPGKEY
pacman-key --lsign-key $GPGKEY

pacman -Syu --noconfirm
pacman -S --noconfirm tmux-cjkwidth htop-vi

ln -sf /usr/share/zoneinfo/Asia/Tokyo /etc/localtime

echo ja_JP.UTF-8 UTF-8 >> /etc/locale.gen
locale-gen

systemctl disable syslog-ng.service
systemctl disable sshd.service
systemctl enable sshd.socket

mkdir -m755 /home/eagletmt
useradd -U eagletmt
gpasswd -a eagletmt wheel
gpasswd -a eagletmt systemd-journal
echo '%wheel ALL=(ALL) NOPASSWD: ALL' >> /etc/sudoers.d/10_wheel
mkdir -m700 ~eagletmt/.ssh
curl https://github.com/eagletmt.keys > ~eagletmt/.ssh/authorized_keys
chmod 600 ~eagletmt/.ssh/authorized_keys
chown -R eagletmt:eagletmt ~eagletmt/.ssh
