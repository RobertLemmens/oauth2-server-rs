# oauth2-server-rs

An oauth2 server build in rust.

## Deployment
### Prerequisites
* A postgres database
### Deployment
There are a couple of ways to deploy this, choose one of the following methods:
1. Kubernetes: Install the helm chart under the helm/ directory. Inspect the values.yaml and deployment.yaml for parameters.
2. Knative: Install the service.yaml under the knative/ directory. Inspect service.yaml for parameters.
  * `kubectl apply -f knative/service.yaml -n your_namespace`
  * `kubectl get ksvc oauth2-tokenserver -n your_namespace` to get the url. 
3. Docker: docker run rlemmens/oauth2-server-rs:0.3.1
  * Make sure to pass environment variables needed (like postgres db settings). See the .env file for all parameters.
4. DIY: Build the container and run it. See the .env file for parameters you need to supply through the environment.
  * The Dockerfile contains a builder so you can just run `docker build -t oauth2server .` without thinking about rust or dependencies.

## Goals
The goal is to implement an oauth2 server based on [RFC-6749](https://tools.ietf.org/html/rfc6749) and [RFC-7662](https://tools.ietf.org/html/rfc7662) in rust. 

### Not implemented yet
Below is a list of RFC functions that dont work yet, and im not sure when ill implement them because i dont need them yet. PR's are always welcome.
* Introspect tokens with Bearer authentication (only client id authentication can introspect tokens)
* Authorization code flow
* Implicit flow
* Refresh tokens

### Things id like to do

* nicer errors (application and response)
* logging / tracing
* bootstrap database if needed

