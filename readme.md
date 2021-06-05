# oauth2-server-rs

An oauth2 server build in rust.

## Goals

The goal is to implement an oauth2 server based on [RFC-6749](https://tools.ietf.org/html/rfc6749) and [RFC-7662](https://tools.ietf.org/html/rfc7662) in rust.

## Not implemented yet
Below is a list of RFC functions that dont work yet, and im not sure when ill implement them because i dont need them yet. PR's are always welcome.
* Introspect tokens with Bearer authentication (only client id authentication can introspect tokens)
* Authorization code flow
* Implicit flow
* Refresh tokens

## Things id like to do

* nicer errors (application and response)
* logging / tracing
* bootstrap database if needed
