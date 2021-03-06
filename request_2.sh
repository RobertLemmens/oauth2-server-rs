#!/bin/bash
curl --user 1234id:1234secret -X POST http://localhost:8081/oauth2/token?client_id\=123\&client_secret\=testclear\&grant_type\=password\&username\=topkek\&password\=topkek
