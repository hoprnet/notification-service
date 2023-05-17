gcp-login:
	gcloud auth configure-docker europe-west3-docker.pkg.dev
	gcloud auth application-default print-access-token | helm registry login -u oauth2accesstoken --password-stdin https://europe-west3-docker.pkg.dev

build:
	npm run build

run:
	npm start

helm-test:
	helm install --dry-run --namespace monitoring --create-namespace -f ./charts/notification-service/values-testing.yaml notification-service ./charts/notification-service/

helm-install:
	helm install --namespace monitoring --create-namespace -f ./charts/notification-service/values-testing.yaml notification-service ./charts/notification-service/

helm-uninstall:
	helm uninstall --namespace monitoring notification-service

helm-upgrade:
	helm upgrade --namespace monitoring --create-namespace -f ./charts/notification-service/values-testing.yaml notification-service ./charts/notification-service/

helm-package:
	helm repo add bitnami https://charts.bitnami.com/bitnami
	helm repo update
	helm dependency build ./charts/notification-service
	helm package ./charts/notification-service --version 1.1.0

helm-publish:
	helm push notification-service-1.1.0.tgz oci://europe-west3-docker.pkg.dev/hoprassociation/helm-charts

docker-build:
	docker build -t gcr.io/hoprassociation/notification-service:latest --platform linux/amd64 --progress plain .

docker-push:
	docker push gcr.io/hoprassociation/notification-service:latest
