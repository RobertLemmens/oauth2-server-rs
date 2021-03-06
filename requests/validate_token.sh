#!/bin/bash
TOKEN=$1
curl --user top:top -d "token=$1" -X POST http://localhost:8081/oauth2/introspect
