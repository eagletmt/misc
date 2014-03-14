#!/usr/bin/env ruby
require 'json'

template = {
  variables: {
    aws_access_key: '',
    aws_secret_key: '',
    region: 'ap-northeast-1',
  },
  builders: [{
    type: 'amazon-ebs',
    access_key: '{{user `aws_access_key`}}',
    secret_key: '{{user `aws_secret_key`}}',
    region: '{{user `region`}}',
    source_ami: 'ami-31d9a830', # https://www.uplinklabs.net/projects/arch-linux-on-ec2/
    instance_type: 't1.micro',
    ssh_username: 'root',
    ami_name: 'packer-ec2-arch64 {{timestamp}}',
  }],
  provisioners: [{
    type: 'shell',
    script: 'provisioner.sh',
  }],
}

puts JSON.pretty_generate(template)
