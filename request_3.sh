#!/bin/bash
curl --user 12345id:12345secret -X POST http://localhost:8081/oauth/token?client_id\=123\&client_secret\=testclear\&grant_type\=password\&username\=test\&password\=test
