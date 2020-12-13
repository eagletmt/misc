# s3-prefix-get
Download all S3 objects under the given prefix.

s3-prefix-get is similar to `aws s3 cp --recursive`, but it doesn't treat `/` as a delimiter.
It's useful for downloading non-awscli-friendly objects such as S3 access logs, CloudFront access logs, etc..
