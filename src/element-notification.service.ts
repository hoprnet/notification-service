import * as https from 'https'
import {v4 as uuidv4} from 'uuid';
import debug from 'debug';

const debugLog: debug.IDebugger = debug('notification-service:element-notification-service');

/**
 * Servicec class that wraps the sending of messages to Element rooms
 */
export class ElementNotificationService {

    private serverHost: string
    private apiToken: string;

    constructor(apiToken?: string, serverHost?: string) {
        this.apiToken = apiToken || process.env.MATRIX_API_TOKEN || 'empty_token';
        this.serverHost = serverHost || process.env.MATRIX_SERVER_HOST || 'hoprnet.modular.im';
    }

    /**
     * Send a message to the giving room
     * @param roomId The id of the room
     * @param message The contents of the message
     * @returns The id of the message sent to Element
     */
    public async sendMessageToRoom(roomId: string, message: string): Promise<any> {
      return new Promise((resolve, reject) => {
        // Payload of the request to send to Element
        const payload = JSON.stringify({
            msgtype: "m.notice",
            body: message,
            formatted_body: message,
            format: 'org.matrix.custom.html'
        })

        // Id of the message to send
        const messageId = uuidv4();

        // Build Http Request options
        const requestOptions: https.RequestOptions = {
            host: this.serverHost,
            port: 443,
            path: `/_matrix/client/r0/rooms/${roomId}/send/m.room.message/${messageId}`,
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${this.apiToken}`,
                'Accept': 'application/json',
                'Content-Type': 'application/json',
                'Content-Length': Buffer.byteLength(payload)
            }
          };
         
          let managedResponse: any = {};
          // Build Http request object
          const request = https.request(requestOptions, (res) => {
            managedResponse.status = res.statusCode;
            managedResponse.headers = res.headers;
            res.setEncoding('utf8');
            let data: any[] = [];

            // Define upon receive data handler
            res.on('data', (chunk) => {
              data.push(chunk);
            });

            // Define upon response end handler
            res.on('end', () => {
              managedResponse.body = data.join('').toString();
              debugLog(`Response: ${JSON.stringify(managedResponse)}`);
              res.statusCode == 200 ? resolve(`{ "messageId": "${messageId}" }`) : resolve(managedResponse)
            });
          });
          
          // Define upon error handler
          request.on('error', (e) => {
            debugLog(`Response: ${JSON.stringify(managedResponse)}`);            
            debugLog(`problem with request: ${e.message}`);
            reject(e);
          });
          
          // Write payload data into request body
          request.write(payload);
          // Send request
          request.end();
        });

    }

}