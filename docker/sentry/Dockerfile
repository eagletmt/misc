FROM ubuntu:14.04
MAINTAINER Kohei Suzuki <eagletmt@gmail.com>

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
  && apt-get -y upgrade \
  && apt-get install -y python-dev python-pip libxml2-dev libxslt-dev libpq-dev libffi-dev
RUN pip install sentry==7.4.1 psycopg2
ADD sentry.conf.py /root/.sentry/sentry.conf.py
RUN mkdir /log
CMD ["sentry", "start"]
