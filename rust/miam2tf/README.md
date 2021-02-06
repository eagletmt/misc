# miam2tf
Convert [miam](https://github.com/codenize-tools/miam) to Terraform definition (WIP)

## Usage
```
% cargo build   # Requires `rake` to build libmruby.a
% cat IAMfile
Dir.glob('users/*.iam').each do |file|
  require file
end
% target/debug/miam2tf | terraform fmt -
resource "aws_iam_user" "s3viewer" {
  name = "s3viewer"
  path = "/"
}
resource "aws_iam_user_policy" "s3viewer-s3viewer" {
  name   = "s3viewer"
  user   = aws_iam_user.s3viewer.name
  policy = data.aws_iam_policy_document.s3viewer-s3viewer.json
}
data "aws_iam_policy_document" "s3viewer-s3viewer" {
  version = "2012-10-17"
  statement {
    effect    = "Allow"
    actions   = ["s3:Get*", "s3:List*"]
    resources = ["*"]
  }
}
(snip)
```
