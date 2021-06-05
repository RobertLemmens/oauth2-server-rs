#!/bin/bash
curl --user top:top -d "grant_type=password&scope=read+write&username=test&password=test" -X POST http://localhost:8081/oauth2/token
