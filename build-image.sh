#! /usr/bin/env bash

readonly BASEDIR="$(cd "$(dirname "$0")" && pwd)"

docker build $BASEDIR -t nebula-prom-transformer
