minikube kubectl -- apply -f postgresql/configmap.yaml
minikube kubectl -- apply -f postgresql/pg.yaml
minikube kubectl -- apply -f postgresql/secret.yaml

minikube kubectl -- apply -f mosquito/configmap.yaml
minikube kubectl -- apply -f mosquitto/service.yaml
minikube kubectl -- apply -f mosquitto/deployment.yaml

minikube kubectl -- apply -f deployments/user-documentation-test.yaml
minikube kubectl -- apply -f services/user-documentation-test.yaml

minikube kubectl -- apply -f deployments/hub.yaml
minikube kubectl -- apply -f services/hub.yaml

minikube kubectl -- apply -f deployments/fake-device.yaml
minikube kubectl -- apply -f services/fake-device.yaml

minikube kubectl -- apply -f deployments/marketing.yaml
minikube kubectl -- apply -f services/marketing.yaml

minikube kubectl -- apply -f deployments/internal-documentation.yaml
minikube kubectl -- apply -f services/internal-documentation.yaml

minikube kubectl -- apply -f ingress/nginx.yaml

