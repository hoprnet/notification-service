# Notification Service

The goal of this repository is to create a centralized service in charge of handling the notifications with externall tools like Email, Element or SNS. At this moment, only Element notification is supported

## Development configuration

- Create a .env file with the following contents:
```
#DEBUG=*
DEBUG=notification-service*
MATRIX_SERVER_HOST=hoprnet.modular.im
MATRIX_API_TOKEN=<This value can be obtained from the users settings, under the Help & About tab>
PORT=8080
NODE_ENV=development
```
- Compile: `npm run build`
- Start the service: `npm run start:dev`
- Get the id of the room where the user holding the API token has permission to send messages. The room id can be obtained from the Room Settings, under the Advanced tab. Eg: `!wNGkijjxWsgBSbpNih:hoprnet.io` will send messages to `notification-service-testing` channel
- Notice that the roomId starts with the character `!` which needs to be replaced from the curl command with the character `%21`
- Test the service: 
```
curl -H "Content-Type: application/json" -X POST --data "@test/alertmanager-notification.json" http://localhost:8080/alertmanager/room/${roomId}
```

## Pipelines

1. Helm chart README.md file is updated automatically with changes on values.yaml
2. Package version needs to be updated when creating a PR that impact docker image
3. Helm chart version needs to be updated when creating a PR that impact chart files

## License

GPL-3.0