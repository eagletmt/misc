#!/bin/sh
set -ex

echo arch64.local > /etc/hostname

ln -s /usr/share/zoneinfo/Asia/Tokyo /etc/localtime
sed -i.bak -e 's/#\(en_US.UTF-8.*\)/\1/' /etc/locale.gen
rm /etc/locale.gen.bak
locale-gen

# For vagrant
echo -e 'vagrant\nvagrant\n' | passwd
useradd -m vagrant
echo -e 'vagrant\nvagrant\n' | passwd vagrant
echo 'vagrant ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/vagrant
chmod 0440 /etc/sudoers.d/vagrant

# Disable PredictableNetworkInterfaceNames.
# See https://github.com/mitchellh/vagrant/blob/master/plugins/guests/arch/cap/configure_networks.rb
# and http://www.freedesktop.org/wiki/Software/systemd/PredictableNetworkInterfaceNames/#idontlikethishowdoidisablethis
ln -sf /dev/null /etc/udev/rules.d/80-net-name-slot.rules # v197 <= systemd <= v208
ln -sf /dev/null /etc/udev/rules.d/80-net-setup-link.rules  # v209 <= systemd

# For ssh
systemctl enable sshd.service
systemctl enable dhcpcd@eth0.service

mkinitcpio -p linux
grub-install --target=i386-pc --recheck /dev/sda
grub-mkconfig -o /boot/grub/grub.cfg
