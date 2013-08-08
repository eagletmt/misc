#!/bin/sh
set -ex
http_root="$1"
shift

sgdisk --new 1::+1m --typecode 1:ef02 --new 2::+100m --new 3 /dev/sda
mkfs.ext4 /dev/sda2
mkfs.ext4 /dev/sda3
mount /dev/sda3 /mnt
mkdir /mnt/boot
mount /dev/sda2 /mnt/boot

#cp /etc/pacman.d/mirrorlist /tmp/mirrorlist
#rankmirrors -n 3 /tmp/mirrorlist > /etc/pacman.d/mirrorlist
echo 'Server = http://ftp.jaist.ac.jp/pub/Linux/ArchLinux/$repo/os/$arch' > /etc/pacman.d/mirrorlist
pacstrap /mnt base grub sudo openssh
genfstab -U -p /mnt >> /mnt/etc/fstab

setup_chroot=/root/setup-chroot.sh
dhcpcd_txt=/root/dhcpcd.txt
curl -o /mnt/$setup_chroot "$http_root/setup-chroot.sh"
chmod +x /mnt/$setup_chroot
systemctl list-units | awk '/dhcpcd/{print $1}' > /mnt/$dhcpcd_txt
arch-chroot /mnt $setup_chroot

rm /mnt/$setup_chroot
rm /mnt/$dhcpcd_txt
reboot
