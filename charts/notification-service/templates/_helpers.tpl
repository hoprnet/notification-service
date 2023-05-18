
{{/*
Template Secret
*/}}
{{- define "notification-service.secretApiToken" -}}
{{- if .Values.matrix.secretReferenceKey -}}
{{ .Values.matrix.secretReferenceKey }}
{{- else -}}
{{ template "common.names.fullname" . }}
{{- end -}}
{{- end -}}