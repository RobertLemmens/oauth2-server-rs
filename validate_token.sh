#!/bin/bash
TOKEN=$1
curl --user 1234id:1234secret -d "token=$1" -X POST http://localhost:8081/oauth2/introspect
