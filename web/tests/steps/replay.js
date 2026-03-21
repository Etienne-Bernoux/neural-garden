import { createBdd } from 'playwright-bdd';
import { expect } from '@playwright/test';

const { Given, When, Then } = createBdd();

Given('le viewer est ouvert', async ({ page }) => {
    // Ecouter les erreurs console pour debug
    page.on('pageerror', err => console.log('PAGE ERROR:', err.message));

    // Naviguer et attendre que le montage soit charge
    const [response] = await Promise.all([
        page.waitForResponse(resp => resp.url().includes('montage.json'), { timeout: 10000 }),
        page.goto('/'),
    ]);

    // Verifier que le montage a ete servi correctement
    expect(response.status()).toBe(200);

    // Attendre que le canvas soit rendu
    await page.waitForSelector('#canvas', { timeout: 10000 });

    // Attendre que le clip info soit mis a jour (preuve que le montage est charge)
    await page.waitForFunction(
        () => {
            const el = document.getElementById('clip-info');
            return el && el.textContent !== 'Clip 0/0';
        },
        { timeout: 10000 },
    );

    // Laisser Three.js s'initialiser et l'auto-play commencer
    await page.waitForTimeout(500);
});

Then('le canvas Three.js est visible', async ({ page }) => {
    const canvas = page.locator('#canvas');
    await expect(canvas).toBeVisible();
});

Then('le panneau de stats est visible', async ({ page }) => {
    const panel = page.locator('#panel');
    await expect(panel).toBeVisible();
});

Then('la population affichée est supérieure à 0', async ({ page }) => {
    await page.waitForFunction(
        () => {
            const el = document.getElementById('population');
            return el && parseInt(el.textContent) > 0;
        },
        { timeout: 10000 },
    );
    const popText = await page.locator('#population').textContent();
    expect(parseInt(popText)).toBeGreaterThan(0);
});

Then('la saison est affichée', async ({ page }) => {
    const seasonEl = page.locator('#season');
    const text = await seasonEl.textContent();
    expect(['Spring', 'Summer', 'Autumn', 'Winter', '-']).toContain(text);
});

When('je clique sur le bouton play', async ({ page }) => {
    // L'app auto-play au chargement. On attend que la timeline finisse,
    // puis on clique play pour relancer.
    // D'abord, on pause si c'est en cours
    const btnText = await page.locator('#btn-play').textContent();
    if (btnText === '\u23F8') {
        // Deja en train de jouer, on le laisse
        return;
    }
    // Sinon on clique pour relancer
    await page.locator('#btn-play').click();
});

When(/^j'attends (\d+) secondes$/, async ({ page }, seconds) => {
    await page.waitForTimeout(parseInt(seconds) * 1000);
});

Then('le tick affiché est supérieur à 0', async ({ page }) => {
    // Attendre que le tick soit > 0 (avec retry car l'animation est asynchrone)
    await page.waitForFunction(
        () => {
            const el = document.getElementById('tick');
            return el && parseInt(el.textContent) > 0;
        },
        { timeout: 10000 },
    );
    const tickText = await page.locator('#tick').textContent();
    const tick = parseInt(tickText);
    expect(tick).toBeGreaterThan(0);
});

Then(/^l'info clip affiche "(.+)"$/, async ({ page }, info) => {
    const clipInfo = page.locator('#clip-info');
    await expect(clipInfo).toHaveText(info);
});

When('je clique sur le canvas', async ({ page }) => {
    const canvas = page.locator('#canvas');
    const box = await canvas.boundingBox();
    if (box) {
        // Clic au centre du canvas
        await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
    }
});

Then('le panneau de la plante sélectionnée est visible ou caché', async ({ page }) => {
    // Le panneau plant-info existe dans le DOM (peut etre visible ou non selon le clic)
    const plantInfo = page.locator('#plant-info');
    await expect(plantInfo).toBeAttached();
});
