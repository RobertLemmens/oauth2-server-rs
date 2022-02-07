#!/bin/bash
if [[ $1 == 'dev' ]]
then
echo "Building dev image"
docker build -t rlemmens/oauth2-server-rs:dev .
docker push rlemmens/oauth2-server-rs:dev
else
docker build -t rlemmens/oauth2-server-rs:0.2.0 .
docker push rlemmens/oauth2-server-rs:0.2.0
fi