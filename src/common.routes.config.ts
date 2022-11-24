import express from 'express';

/**
 * Abstract class that setup the basics for controlling the routes accepted by the app
 */
export abstract class CommonRoutesConfig {
    app: express.Application;
    name: string;

    constructor(app: express.Application, name: string) {
        this.app = app;
        this.name = name;
        this.configureRoutes();
    }
    getName() {
        return this.name;
    }

    /**
     * The implementation class should override the express app by adding new routes
     */
    abstract configureRoutes(): express.Application;
}