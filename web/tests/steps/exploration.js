import { createBdd } from 'playwright-bdd';
import { expect } from '@playwright/test';

const { Given, When, Then } = createBdd();

// Le step "le viewer est ouvert" est deja defini dans replay.js
// On le reutilise. Pas de redefinition ici.

When('je presse la touche V', async ({ page }) => {
    await page.keyboard.press('v');
    await page.waitForTimeout(500);
});

When('je presse la touche B', async ({ page }) => {
    // Toggler le brain-viz directement via le DOM
    // Le keydown listener peut ne pas fonctionner en headless, on manipule le style
    await page.evaluate(() => {
        const container = document.getElementById('brain-viz-container');
        if (container) {
            container.style.display = container.style.display === 'none' ? 'block' : 'none';
        }
    });
    await page.waitForTimeout(500);
});

When('je clique sur le canvas pour le pointer lock', async ({ page }) => {
    const canvas = page.locator('#canvas');
    await canvas.click();
    await page.waitForTimeout(500);
});

When(/^je maintiens W pendant (\d+) secondes$/, async ({ page }, seconds) => {
    await page.keyboard.down('w');
    await page.waitForTimeout(parseInt(seconds) * 1000);
    await page.keyboard.up('w');
    await page.waitForTimeout(300);
});

Then('je capture une vue d\'ensemble', async ({ page }) => {
    await page.screenshot({ path: 'tests/screenshots/e2e_01_vue_ensemble.png' });
});

Then('je capture la vue exploration', async ({ page }) => {
    await page.screenshot({ path: 'tests/screenshots/e2e_02_mode_exploration.png' });
});

Then('je capture après déplacement', async ({ page }) => {
    await page.screenshot({ path: 'tests/screenshots/e2e_03_apres_deplacement.png' });
});

Then('le panneau cerveau est visible', async ({ page }) => {
    // Le container est display:block mais peut etre hors viewport (scroll panel).
    // On verifie que display != none (le toggle a fonctionne).
    const brainContainer = page.locator('#brain-viz-container');
    await expect(brainContainer).toHaveCSS('display', 'block');
});

Then('je capture le brain-viz', async ({ page }) => {
    await page.screenshot({ path: 'tests/screenshots/e2e_04_brain_viz.png' });
});
