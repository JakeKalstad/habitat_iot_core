minikube kubectl delete deployment habitat-fake-device
minikube kubectl delete service habitat-fake-device

minikube image load habitat-fake-device:local
minikube kubectl -- apply -f ../devops/kubernetes/deployments/fake-device.yaml
minikube kubectl -- apply -f ../devops/kubernetes/services/fake-device.yaml
