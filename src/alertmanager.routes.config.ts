import {CommonRoutesConfig} from './common.routes.config';
import express from 'express';
import debug from 'debug';
import { ParentAlert } from './alertmanager/parent-alert.class';
import { ChildAlert } from './alertmanager/child-alert.class';
import { ElementNotificationService } from './element-notification.service';

const moment = require('moment');
const debugLog: debug.IDebugger = debug('notification-service:alertmanager-routes');
const GITHUB_URL_IMAGE_BASE= "https://github.com/hoprnet/hopr-devops/raw/feature/75-alertmamanger/docs/alerts";

/**
 * Configure a route to handle AlertManager alerts and notify them via Element 
 */
export class AlertManagerRoutes extends CommonRoutesConfig {

    private elementNotifier: ElementNotificationService;

    constructor(app: express.Application) {
        super(app, 'AlertManagerRoutes');
        this.elementNotifier = new ElementNotificationService()
    }

    /**
     * Given an array of alerts, determine which alert has highest priority
     * @param alerts Array of alerts
     * @returns the name of the highest serverity in the array
     */
    private getHighestSeverity(alerts: ChildAlert[]): string {
        const uniqueSeverities = alerts.map(alert => alert.labels.severity).filter(n => n).toString();
        if(uniqueSeverities.indexOf('critical') >= 0){
            return 'critical';
        }
        if(uniqueSeverities.indexOf('warning') >= 0){
            return 'warning';
        }
        if(uniqueSeverities.indexOf('info') >= 0){
            return 'info';
        }                        
        return 'unknown'
    }

    /**
     * Parses an ChildAlert into an HTML row
     * @param alert the contents of the alert
     * @returns The HTML table row contents with the child alert
     */
    private parseChildAlert(alert: ChildAlert): string[] {
        let tableRowLines: string[] = [];
        let startedDate = (moment(alert.startsAt)).format('HH:mm:ss')
        tableRowLines.push('\t\t\t<tr>');
        tableRowLines.push(`\t\t\t\t<td><a href="${alert.annotations.runbook_url}" target="_blank">${alert.labels.alertname}</a></td>`);
        tableRowLines.push(`\t\t\t\t<td><img src="${GITHUB_URL_IMAGE_BASE}/${alert.labels.severity}.png"/></td>`);
        tableRowLines.push(`\t\t\t\t<td>${startedDate}</td>`);
        tableRowLines.push(`\t\t\t\t<td>${alert.annotations.description}</td>`);
        tableRowLines.push('\t\t\t</tr>');        
        return tableRowLines;
    }

    /**
     * Parses the severity, summary and description
     * @param parentAlert Parent alert
     * @returns Html heading lines
     */
    parseHeadingAlert (parentAlert: ParentAlert): string[] {
        let headingLines: string[] = [];

        // Severity
        headingLines.push(`\t<p>\n\t\t<img src="${GITHUB_URL_IMAGE_BASE}/${this.getHighestSeverity(parentAlert.alerts)}.png"/>\n\t</p>`);

        // Summary
        if( parentAlert.commonAnnotations.summary ) {
            headingLines.push(`\t<h2>${parentAlert.commonAnnotations.summary}</h2>`);
        } else if (parentAlert.commonAnnotations.summary_group ) {
            headingLines.push(`\t<h2>${parentAlert.commonAnnotations.summary_group}</h2>`);
        } else {
            headingLines.push('\t<h4>[ monitoring ][ infrastructure ] A general alert has raisen</h4>');
        }

        // Description
        if( parentAlert.commonAnnotations.description ) {
            headingLines.push(`\t<p>${parentAlert.commonAnnotations.description}</p>`);
        } else if (parentAlert.commonAnnotations.description_group ) {
            headingLines.push(`\t<p>${parentAlert.commonAnnotations.description_group}</p>`);
        } else {
            headingLines.push('\t<p></p>');
        }

        return headingLines;
    }

    /**
     * Builds the HTML table with child alerts
     * @param alerts array of alerts
     * @returns html table
     */
    private parseChildAlerts(alerts: ChildAlert[]): string[] {
        let tableLines: string[] = [];
        tableLines.push('\t<table>');
        tableLines.push('\t\t<thead>');
        tableLines.push('\t\t\t<td>Name</td>');
        tableLines.push('\t\t\t<td>Severity</td>');
        tableLines.push('\t\t\t<td>Started</td>');
        tableLines.push('\t\t\t<td>Description</td>');
        tableLines.push('\t\t</thead>');
        tableLines.push('\t\t<tbody>');
        alerts.forEach((alert: ChildAlert) => tableLines.push(...this.parseChildAlert(alert)));
        tableLines.push('\t\t</tbody>');
        tableLines.push('\t</table>'); 
        return tableLines;       
    }

    /**
     * Parse parent alert into HTML
     * @param parentAlert parent alert
     * @returns Main HTML representing the parent alert
     */
    private parseParentAlert(parentAlert: ParentAlert): string {
        let htmlLines: string[] = [];
        htmlLines.push('<html>');
        htmlLines.push('<body>');
        htmlLines.push(...this.parseHeadingAlert(parentAlert));
        htmlLines.push(...this.parseChildAlerts(parentAlert.alerts));
        htmlLines.push('</body>');
        htmlLines.push('</html>');        
        return htmlLines.join('\n');
    }

    
    public configureRoutes() {
        this.app.route(`/alertmanager/room/:roomId`)
            .post((req: express.Request, res: express.Response) => {
                const roomId = req.params.roomId;
                if (roomId == undefined) {
                    res.status(500).json({ error: "roomId param not provided in url" });
                } else {
                    try{ 
                        debugLog(`Message received with body: ${JSON.stringify(req.body)}`);
                        const notification = this.parseParentAlert(req.body as ParentAlert);
                        debugLog(`Notification sent with content:\n${notification}`)
                        this.elementNotifier.sendMessageToRoom(roomId, notification).then((messageId: string) => {
                            res.status(200).json({ messageId })                        
                        });
                    } catch(err: any) {
                        debugLog(JSON.stringify(err))
                        res.status(500).json(JSON.stringify(err));
                    }
                }
            });

        return this.app;
    }
}