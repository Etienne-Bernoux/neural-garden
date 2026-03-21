// Script de debug : capture des screenshots du viewer en mode live
// Usage : node tests/debug-screenshots.js
// Prerequis : garden live doit tourner sur localhost:3000

import { chromium } from '@playwright/test';

const SCREENSHOTS_DIR = 'tests/screenshots';

async function captureScreenshots() {
    const browser = await chromium.launch({
        headless: true,
        args: ['--use-gl=angle', '--use-angle=swiftshader'],
    });
    const page = await browser.newPage({ viewport: { width: 1280, height: 720 } });

    console.log('Connexion au viewer...');
    await page.goto('http://localhost:3000');

    // Attendre que le canvas soit la
    await page.waitForSelector('#canvas', { timeout: 10000 });

    // Screenshot 1 : etat initial (juste apres chargement)
    await page.waitForTimeout(2000);
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/01_initial.png`, fullPage: false });
    console.log('Screenshot 1 : etat initial');

    // Screenshot 2 : apres 3 secondes (quelques ticks)
    await page.waitForTimeout(3000);
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/02_apres_3s.png`, fullPage: false });
    console.log('Screenshot 2 : apres 3 secondes');

    // Screenshot 3 : apres 6 secondes
    await page.waitForTimeout(3000);
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/03_apres_6s.png`, fullPage: false });
    console.log('Screenshot 3 : apres 6 secondes');

    // Screenshot 4 : apres 10 secondes
    await page.waitForTimeout(4000);
    await page.screenshot({ path: `${SCREENSHOTS_DIR}/04_apres_10s.png`, fullPage: false });
    console.log('Screenshot 4 : apres 10 secondes');

    // Capturer aussi les logs console
    const logs = [];
    page.on('console', msg => logs.push(`[${msg.type()}] ${msg.text()}`));

    // Screenshot 5 : capturer le panneau stats
    const stats = await page.evaluate(() => {
        return {
            tick: document.getElementById('tick')?.textContent,
            season: document.getElementById('season')?.textContent,
            population: document.getElementById('population')?.textContent,
            fitness: document.getElementById('fitness')?.textContent,
            clipInfo: document.getElementById('clip-info')?.textContent,
        };
    });
    console.log('Stats du panneau :', JSON.stringify(stats, null, 2));

    // Screenshot 6 : capturer les erreurs JS
    const errors = await page.evaluate(() => {
        return window.__errors || [];
    });
    if (errors.length > 0) {
        console.log('Erreurs JS :', errors);
    }

    console.log('Logs console :', logs.slice(0, 20));

    await browser.close();
    console.log(`Screenshots sauvegardees dans ${SCREENSHOTS_DIR}/`);
}

// Creer le dossier
import { mkdirSync } from 'fs';
try { mkdirSync(`${SCREENSHOTS_DIR}`, { recursive: true }); } catch {}

captureScreenshots().catch(console.error);
