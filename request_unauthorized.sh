#!/bin/bash
curl -i -X POST http://localhost:8081/oauth2/token?client_id\=123\&client_secret\=testclear\&grant_type\=password\&username\=test\&password\=wrongpassword
