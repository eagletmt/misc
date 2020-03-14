resource "aws_s3_bucket" "gyazo" {
  bucket = "gyazo.wanko.cc"
  website {
    index_document = "index.html"
  }
  cors_rule {
    allowed_headers = ["*"]
    allowed_methods = ["GET"]
    allowed_origins = ["*"]
    max_age_seconds = 3000
  }
}

resource "aws_acm_certificate" "gyazo" {
  provider          = aws.use1
  domain_name       = "gyazo.wanko.cc"
  validation_method = "DNS"
}

data "aws_route53_zone" "wanko-cc" {
  name         = "wanko.cc."
  private_zone = false
}

resource "aws_route53_record" "gyazo-validation" {
  zone_id = data.aws_route53_zone.wanko-cc.id
  name    = aws_acm_certificate.gyazo.domain_validation_options[0].resource_record_name
  type    = aws_acm_certificate.gyazo.domain_validation_options[0].resource_record_type
  records = [aws_acm_certificate.gyazo.domain_validation_options[0].resource_record_value]
  ttl     = 60
}

resource "aws_acm_certificate_validation" "gyazo" {
  provider                = aws.use1
  certificate_arn         = aws_acm_certificate.gyazo.arn
  validation_record_fqdns = [aws_route53_record.gyazo-validation.fqdn]
}

locals {
  s3_origin_id = "s3-gyazo.wanko.cc"
}

resource "aws_cloudfront_distribution" "gyazo" {
  origin {
    domain_name = aws_s3_bucket.gyazo.bucket_domain_name
    origin_id   = local.s3_origin_id
  }

  enabled         = true
  is_ipv6_enabled = true
  aliases         = [aws_acm_certificate.gyazo.domain_name]
  default_cache_behavior {
    allowed_methods  = ["GET", "HEAD"]
    cached_methods   = ["GET", "HEAD"]
    target_origin_id = local.s3_origin_id
    forwarded_values {
      query_string = false
      cookies {
        forward = "none"
      }
    }
    viewer_protocol_policy = "redirect-to-https"
    min_ttl                = 0
    default_ttl            = 24 * 60 * 60
    max_ttl                = 365 * 24 * 60 * 60
  }

  price_class = "PriceClass_All"
  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }
  viewer_certificate {
    acm_certificate_arn      = aws_acm_certificate.gyazo.arn
    ssl_support_method       = "sni-only"
    minimum_protocol_version = "TLSv1.1_2016"
  }
}

resource "aws_route53_record" "gyazo" {
  for_each = toset(["A", "AAAA"])

  zone_id = data.aws_route53_zone.wanko-cc.id
  name    = "gyazo.wanko.cc"
  type    = each.value
  alias {
    name                   = aws_cloudfront_distribution.gyazo.domain_name
    zone_id                = aws_cloudfront_distribution.gyazo.hosted_zone_id
    evaluate_target_health = false
  }
}

