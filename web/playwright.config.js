import { defineConfig } from '@playwright/test';
import { defineBddConfig } from 'playwright-bdd';

const testDir = defineBddConfig({
    features: 'tests/features/*.feature',
    steps: 'tests/steps/*.js',
});

export default defineConfig({
    testDir,
    timeout: 30000,
    use: {
        baseURL: 'http://localhost:3333',
        headless: true,
        launchOptions: {
            args: [
                '--use-gl=angle',
                '--use-angle=swiftshader',
                '--enable-webgl',
                '--ignore-gpu-blocklist',
            ],
        },
    },
    webServer: {
        command: 'node tests/test-server.js',
        port: 3333,
        timeout: 10000,
        reuseExistingServer: false,
    },
});
