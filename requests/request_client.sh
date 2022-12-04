#!/bin/bash
curl --user top:top_321 -d "grant_type=client_credentials" -X POST http://localhost:8081/oauth2/token
