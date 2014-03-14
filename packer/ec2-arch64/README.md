# ec2-arch64
Create a basic AMI for me.

## Build
```sh
vim variables.json
packer build template.json -var-file variables.json
```

## Update template
```sh
vim template.rb
./template.rb > template.json
```
