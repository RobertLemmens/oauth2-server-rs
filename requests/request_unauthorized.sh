#!/bin/bash
curl -i -d "grant_type=password&username=wrong&password=wrong" -X POST http://localhost:8081/oauth2/token
