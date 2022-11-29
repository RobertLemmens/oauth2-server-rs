#!/bin/bash
docker build -t rlemmens/oauth2-server-rs:0.3.1 .
docker push rlemmens/oauth2-server-rs:0.3.1
