#!/bin/bash

ts=$(date +%s)
ts=$[ts - ts%(5*60)]
prev_ts=$[ts - 5*60]
ts=$(date --date @$ts +%Y%m%d%H%M)
prev_ts=$(date --date @$prev_ts +%Y%m%d%H%M)

BASE_URL='http://tokyo-ame.jwa.or.jp'
fname=amesh-$ts.png
convert \
  <(curl -s $BASE_URL/map/map000.jpg) \
  <(curl -sf $BASE_URL/mesh/000/$ts.gif || curl -sf $BASE_URL/mesh/000/$prev_ts.gif) -composite \
  <(curl -s $BASE_URL/map/msk000.png) -composite \
  $fname && echo $fname
