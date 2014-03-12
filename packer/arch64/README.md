# Packer template for ArchLinux Vagrant box

- http://www.packer.io/
- http://www.vagrantup.com/

```sh
packer build template.json
mv packer_virtualbox-iso_virtualbox.box /tmp/arch64.box
vagrant up
```
