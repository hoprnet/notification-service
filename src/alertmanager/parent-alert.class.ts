import { ChildAlert } from "./child-alert.class";

/**
 * Parent Alert
 */
export class ParentAlert  {

    public receiver: string;
    public status: string;
    public alerts: ChildAlert[];
    public groupLabels: {[key: string]: string};
    public commonLabels: {[key: string]: string};
    public commonAnnotations: {[key: string]: string};
    

}