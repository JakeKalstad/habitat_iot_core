# Kubernetes

## Deployments

Holds the deployment files for all the running services

## Ingress 

Holds the ingess configurations for nginx to serve the different backend services

## Services

Holds the service configurations for individual back ends and their different ports/environment settings

### Commands

if you've already built the docker image for the marketing 

`minikube image load habitat-marketing:local`

`minikube image load internal-documentation:local`

`minikube kubectl -- apply -f deployments/marketing.yaml`

`minikube kubectl -- apply -f services/marketing.yaml`

`minikube kubectl -- apply -f deployments/internal-documentation.yaml`

`minikube kubectl -- apply -f services/internal-documentation.yaml`

`minikube kubectl -- apply -f deployments/hub.yaml`

`minikube kubectl -- apply -f services/hub.yaml`

`minikube kubectl -- apply -f deployments/user-documentation-test.yaml`

`minikube kubectl -- apply -f services/user-documentation-test.yaml`

`minikube kubectl -- apply -f ingress/nginx.yaml`

get the minikube nginx IP address via

`kubectl get ingress`

may take a minute to come up, copy paste the ADDRESS ip value for nginx

add the following line to your `/etc/hosts` file

> 192.168.39.214 habitat-marketing.example

now you can browse to "habitat-marketing.example" on your machine and it will find your minikube nginx/service pod instances