<!--- app-name: Notification Service -->

# Notification Service Chart

This chart packages all the kubernetes resources needed to publish the notification service on Kubernetes

## Installing

```console
$ helm repo add notification-service git+https://github.com/hoprnet/notification-service@deploy/charts?ref=master
$ helm install notification-service notification-service/notification-service
```

## Uninstalling the Chart

To uninstall/delete the release:

```console
helm delete notification-service
```

The command removes all the Kubernetes components associated with the chart and deletes the release.

## Creating a pull request

Chart version `Chart.yaml` should be increased according to [semver](http://semver.org/)


## Parameters

### Common parameters

| Name                | Description                                        | Value           |
| ------------------- | -------------------------------------------------- | --------------- |
| `nameOverride`      | String to partially override common.names.fullname | `""`            |
| `fullnameOverride`  | String to fully override common.names.fullname     | `""`            |
| `namespaceOverride` | String to fully override common.names.namespace    | `""`            |
| `commonLabels`      | Labels to add to all deployed objects              | `{}`            |
| `commonAnnotations` | Annotations to add to all deployed objects         | `{}`            |
| `clusterDomain`     | Kubernetes cluster domain name                     | `cluster.local` |


### Notification Service Parameters

| Name                                                | Description                                                                                                              | Value                                  |
| --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------- |
| `image.registry`                                    | Image registry                                                                                                           | `gcr.io`                               |
| `image.repository`                                  | Image repository                                                                                                         | `hoprassociation/notification-service` |
| `image.tag`                                         | Image tag (immutable tags are recommended)                                                                               | `latest`                               |
| `image.digest`                                      | Image digest in the way sha256:aa.... Please note this parameter, if set, will override the tag                          | `""`                                   |
| `image.pullPolicy`                                  | Image pull policy                                                                                                        | `IfNotPresent`                         |
| `image.pullSecrets`                                 | Image pull secrets                                                                                                       | `[]`                                   |
| `containerPorts.http`                               | Controller HTTP container port to open                                                                                   | `8080`                                 |
| `resources.limits`                                  | The resources limits for the containers                                                                                  | `{}`                                   |
| `resources.requests`                                | The requested resources for the containers                                                                               | `{}`                                   |
| `livenessProbe.enabled`                             | Enable livenessProbe on containers                                                                                       | `true`                                 |
| `livenessProbe.initialDelaySeconds`                 | Initial delay seconds for livenessProbe                                                                                  | `5`                                    |
| `livenessProbe.periodSeconds`                       | Period seconds for livenessProbe                                                                                         | `10`                                   |
| `livenessProbe.timeoutSeconds`                      | Timeout seconds for livenessProbe                                                                                        | `1`                                    |
| `livenessProbe.failureThreshold`                    | Failure threshold for livenessProbe                                                                                      | `3`                                    |
| `livenessProbe.successThreshold`                    | Success threshold for livenessProbe                                                                                      | `1`                                    |
| `readinessProbe.enabled`                            | Enable readinessProbe on containers                                                                                      | `true`                                 |
| `readinessProbe.initialDelaySeconds`                | Initial delay seconds for readinessProbe                                                                                 | `5`                                    |
| `readinessProbe.periodSeconds`                      | Period seconds for readinessProbe                                                                                        | `10`                                   |
| `readinessProbe.timeoutSeconds`                     | Timeout seconds for readinessProbe                                                                                       | `1`                                    |
| `readinessProbe.failureThreshold`                   | Failure threshold for readinessProbe                                                                                     | `3`                                    |
| `readinessProbe.successThreshold`                   | Success threshold for readinessProbe                                                                                     | `1`                                    |
| `startupProbe.enabled`                              | Enable startupProbe on containers                                                                                        | `false`                                |
| `startupProbe.initialDelaySeconds`                  | Initial delay seconds for startupProbe                                                                                   | `10`                                   |
| `startupProbe.periodSeconds`                        | Period seconds for startupProbe                                                                                          | `10`                                   |
| `startupProbe.timeoutSeconds`                       | Timeout seconds for startupProbe                                                                                         | `1`                                    |
| `startupProbe.failureThreshold`                     | Failure threshold for startupProbe                                                                                       | `15`                                   |
| `startupProbe.successThreshold`                     | Success threshold for startupProbe                                                                                       | `1`                                    |
| `customLivenessProbe`                               | Custom livenessProbe that overrides the default one                                                                      | `{}`                                   |
| `customReadinessProbe`                              | Custom readinessProbe that overrides the default one                                                                     | `{}`                                   |
| `customStartupProbe`                                | Custom startupProbe that overrides the default one                                                                       | `{}`                                   |
| `podSecurityContext.enabled`                        | Enabled pods' Security Context                                                                                           | `true`                                 |
| `podSecurityContext.fsGroup`                        | Set pod's Security Context fsGroup                                                                                       | `1001`                                 |
| `podSecurityContext.seccompProfile.type`            | Set pod's Security Context seccompProfile type                                                                           | `RuntimeDefault`                       |
| `containerSecurityContext.enabled`                  | Enabled containers' Security Context                                                                                     | `true`                                 |
| `containerSecurityContext.allowPrivilegeEscalation` | Whether the container can escalate privileges                                                                            | `false`                                |
| `containerSecurityContext.capabilities.drop`        | Which privileges to drop in the container                                                                                | `["ALL"]`                              |
| `containerSecurityContext.readOnlyRootFilesystem`   | Whether the container has a read-only root filesystem                                                                    | `true`                                 |
| `containerSecurityContext.runAsNonRoot`             | Indicates that the container must run as a non-root user                                                                 | `true`                                 |
| `containerSecurityContext.runAsUser`                | Set containers' Security Context runAsUser                                                                               | `1001`                                 |
| `containerSecurityContext.seccompProfile.type`      | Set container's Security Context seccompProfile type                                                                     | `RuntimeDefault`                       |
| `podLabels`                                         | Extra labels for pods                                                                                                    | `{}`                                   |
| `podAnnotations`                                    | Annotations for pods                                                                                                     | `{}`                                   |
| `podAffinityPreset`                                 | Pod affinity preset. Ignored if `affinity` is set. Allowed values: `soft` or `hard`                                      | `""`                                   |
| `podAntiAffinityPreset`                             | Pod anti-affinity preset. Ignored if `affinity` is set. Allowed values: `soft` or `hard`                                 | `soft`                                 |
| `nodeAffinityPreset.type`                           | Node affinity preset type. Ignored if `affinity` is set. Allowed values: `soft` or `hard`                                | `""`                                   |
| `nodeAffinityPreset.key`                            | Node label key to match. Ignored if `affinity` is set                                                                    | `""`                                   |
| `nodeAffinityPreset.values`                         | Node label values to match. Ignored if `affinity` is set                                                                 | `[]`                                   |
| `affinity`                                          | Affinity for pods assignment                                                                                             | `{}`                                   |
| `nodeSelector`                                      | Node labels for pods assignment                                                                                          | `{}`                                   |
| `tolerations`                                       | Tolerations for pods assignment                                                                                          | `[]`                                   |
| `updateStrategy.type`                               | Deployment strategy type                                                                                                 | `RollingUpdate`                        |
| `priorityClassName`                                 | pods' priorityClassName                                                                                                  | `""`                                   |
| `topologySpreadConstraints`                         | Topology Spread Constraints for pod assignment spread across your cluster among failure-domains. Evaluated as a template | `[]`                                   |
| `schedulerName`                                     | Name of the k8s scheduler (other than default) for pods                                                                  | `""`                                   |
| `terminationGracePeriodSeconds`                     | Seconds the pod needs to terminate gracefully                                                                            | `""`                                   |
| `lifecycleHooks`                                    | for the container(s) to automate configuration before or after startup                                                   | `{}`                                   |
| `extraEnvVars`                                      | Array with extra environment variables to add to nodes                                                                   | `[]`                                   |
| `extraEnvVarsCM`                                    | Name of existing ConfigMap containing extra env vars for nodes                                                           | `""`                                   |
| `extraEnvVarsSecret`                                | Name of existing Secret containing extra env vars for nodes                                                              | `""`                                   |


### Traffic Exposure Parameters

| Name                               | Description                                                                                           | Value                        |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------- |
| `service.type`                     | service type                                                                                          | `ClusterIP`                  |
| `service.ports.http`               | service HTTP port number                                                                              | `8080`                       |
| `service.ports.name`               | service HTTP port name                                                                                | `http`                       |
| `service.nodePorts.http`           | Node port for HTTP                                                                                    | `""`                         |
| `service.clusterIP`                | service Cluster IP                                                                                    | `""`                         |
| `service.loadBalancerIP`           | service Load Balancer IP                                                                              | `""`                         |
| `service.loadBalancerSourceRanges` | service Load Balancer sources                                                                         | `[]`                         |
| `service.externalTrafficPolicy`    | service external traffic policy                                                                       | `Cluster`                    |
| `service.annotations`              | Additional custom annotations for service                                                             | `{}`                         |
| `service.extraPorts`               | Extra ports to expose in service                                                                      | `[]`                         |
| `service.sessionAffinity`          | Control where client requests go, to the same pod or round-robin                                      | `None`                       |
| `service.sessionAffinityConfig`    | Additional settings for the sessionAffinity                                                           | `{}`                         |
| `ingress.enabled`                  | Enable ingress record generation                                                                      | `false`                      |
| `ingress.pathType`                 | Ingress path type                                                                                     | `ImplementationSpecific`     |
| `ingress.apiVersion`               | Force Ingress API version (automatically detected if not set)                                         | `""`                         |
| `ingress.ingressClassName`         | IngressClass that will be be used to implement the Ingress                                            | `nginx`                      |
| `ingress.hostname`                 | Default host for the ingress record                                                                   | `notification-service.local` |
| `ingress.path`                     | Default path for the ingress record                                                                   | `/`                          |
| `ingress.annotations`              | Additional custom annotations for the ingress record                                                  | `{}`                         |
| `ingress.extraHosts`               | An array with additional hostname(s) to be covered with the ingress record                            | `[]`                         |
| `ingress.extraPaths`               | An array with additional arbitrary paths that may need to be added to the ingress under the main host | `[]`                         |
| `ingress.extraRules`               | Additional rules to be covered with this ingress record                                               | `[]`                         |


### Other Parameters

| Name                                          | Description                                                      | Value                  |
| --------------------------------------------- | ---------------------------------------------------------------- | ---------------------- |
| `rbac.create`                                 | Specifies whether RBAC resources should be created               | `true`                 |
| `rbac.unsealer.rules`                         | Custom RBAC rules to set for unsealer ClusterRole                | `[]`                   |
| `rbac.keyAdmin.rules`                         | Custom RBAC rules to set for key-admin role                      | `[]`                   |
| `rbac.serviceProxier.rules`                   | Custom RBAC rules to set for service-proxier role                | `[]`                   |
| `serviceAccount.create`                       | Specifies whether a ServiceAccount should be created             | `true`                 |
| `serviceAccount.name`                         | The name of the ServiceAccount to use.                           | `notification-service` |
| `serviceAccount.annotations`                  | Additional Service Account annotations (evaluated as a template) | `{}`                   |
| `serviceAccount.automountServiceAccountToken` | Automount service account token for the server service account   | `true`                 |


### Metrics parameters

| Name                                       | Description                                                                      | Value   |
| ------------------------------------------ | -------------------------------------------------------------------------------- | ------- |
| `metrics.serviceMonitor.enabled`           | Specify if a ServiceMonitor will be deployed for Prometheus Operator             | `false` |
| `metrics.serviceMonitor.namespace`         | Namespace in which Prometheus is running                                         | `""`    |
| `metrics.serviceMonitor.labels`            | Extra labels for the ServiceMonitor                                              | `{}`    |
| `metrics.serviceMonitor.annotations`       | Additional ServiceMonitor annotations (evaluated as a template)                  | `{}`    |
| `metrics.serviceMonitor.jobLabel`          | The name of the label on the target service to use as the job name in Prometheus | `""`    |
| `metrics.serviceMonitor.honorLabels`       | honorLabels chooses the metric's labels on collisions with target labels         | `false` |
| `metrics.serviceMonitor.interval`          | Interval at which metrics should be scraped.                                     | `""`    |
| `metrics.serviceMonitor.scrapeTimeout`     | Timeout after which the scrape is ended                                          | `""`    |
| `metrics.serviceMonitor.metricRelabelings` | Specify additional relabeling of metrics                                         | `[]`    |
| `metrics.serviceMonitor.relabelings`       | Specify general relabeling                                                       | `[]`    |
| `metrics.serviceMonitor.selector`          | Prometheus instance selector labels                                              | `{}`    |

