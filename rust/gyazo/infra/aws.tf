provider "aws" {
  version = "2.53.0"
  region  = "ap-northeast-1"
}

provider "aws" {
  version = "2.53.0"
  region  = "us-east-1"
  alias   = "use1"
}

terraform {
  backend "s3" {
    bucket = "terraform-wanko-cc"
    key    = "gyazo.tfstate"
    region = "ap-northeast-1"
  }
}
