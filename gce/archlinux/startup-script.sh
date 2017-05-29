#!/bin/bash
set -ex

# Prepare root volume
sgdisk --new 1::+1m --typecode 1:ef02 --new 2 /dev/sdb
mkfs.btrfs /dev/sdb2

# Unarchive bootstrap tree
curl -vL -o archlinux-bootstrap.tar.gz https://mirrors.kernel.org/archlinux/iso/latest/archlinux-bootstrap-2017.05.01-x86_64.tar.gz
tar xf archlinux-bootstrap.tar.gz

# pacman-key requires haveged
apt install -y haveged

cat <<'EOS' > root.x86_64/build.sh
#!/bin/bash
set -ex

mount /dev/sdb2 /mnt
trap 'umount /mnt' EXIT

pacman-key --init
pacman-key --populate archlinux

echo 'Server = http://ftp.jaist.ac.jp/pub/Linux/ArchLinux/$repo/os/$arch' >> /etc/pacman.d/mirrorlist
echo 'Server = http://ftp.tsukuba.wide.ad.jp/Linux/archlinux/$repo/os/$arch' >> /etc/pacman.d/mirrorlist

pacstrap /mnt base grub openssh sudo

curl -vL -o /mnt/gce.pkg.tar.xz http://arch.wanko.cc/aur-eagletmt/os/x86_64/gce-compute-image-packages-20170523-2-any.pkg.tar.xz
arch-chroot /mnt pacman --noconfirm -U /gce.pkg.tar.xz
rm /mnt/gce.pkg.tar.xz

echo -e '[Match]\nName=ens4\n\n[Network]\nDHCP=ipv4' > /mnt/etc/systemd/network/ens4.network
ln -sfn /run/systemd/resolve/resolv.conf /mnt/etc/resolv.conf

arch-chroot /mnt systemctl enable \
  sshd.socket \
  systemd-networkd.service \
  systemd-resolved.service \
  google-accounts-daemon.service \
  google-clock-skew-daemon.service \
  google-instance-setup.service \
  google-ip-forwarding-daemon.service \
  google-network-setup.service \
  google-shutdown-scripts.service \
  google-startup-scripts.service

arch-chroot /mnt grub-install --target=i386-pc /dev/sdb
# mkinitcpio returns non-zero code for btrfs, but it's OK.
arch-chroot /mnt mkinitcpio -p linux || true
# https://cloud.google.com/compute/docs/images/import-existing-image#configure_bootloader
echo 'GRUB_CMDLINE_LINUX_DEFAULT="console=ttyS0,38400n8d"' >> /mnt/etc/default/grub
arch-chroot /mnt grub-mkconfig -o /boot/grub/grub.cfg
EOS
chmod +x root.x86_64/build.sh

root.x86_64/usr/bin/arch-chroot root.x86_64 /build.sh
