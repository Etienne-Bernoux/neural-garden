import { createServer } from 'http';
import { readFileSync, existsSync } from 'fs';
import { join, extname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = fileURLToPath(new URL('.', import.meta.url));
const webDir = join(__dirname, '..');

const MIME_TYPES = {
    '.html': 'text/html',
    '.css': 'text/css',
    '.js': 'application/javascript',
    '.json': 'application/json',
};

// Montage mock pour les tests
const mockMontage = {
    version: 1,
    metadata: { total_ticks: 500 },
    clips: [{
        clip: { trigger: 'test', tick_start: 0, tick_end: 100, score: 1.0 },
        header: {
            grid_size: 16,
            altitude: Array.from({ length: 16 }, (_, y) =>
                Array.from({ length: 16 }, (_, x) => {
                    const dx = x - 8, dy = y - 8;
                    return Math.max(0, 0.8 - Math.sqrt(dx*dx + dy*dy) / 10);
                })
            ),
            plants: [
                { id: 1, lineage_id: 0, cells: [[8, 8]], vitality: 80, energy: 50, traits: { hidden_size: 8, exudate_type: 'carbon', carbon_nitrogen_ratio: 0.5, max_size: 20 } },
                { id: 2, lineage_id: 1, cells: [[10, 8]], vitality: 60, energy: 30, traits: { hidden_size: 10, exudate_type: 'nitrogen', carbon_nitrogen_ratio: 0.7, max_size: 25 } },
            ],
        },
        events: [
            { t: 1, e: 'germinate', p: 1 },
            { t: 2, e: 'germinate', p: 2 },
            { t: 5, e: 'grow', p: 1, x: 9, y: 8 },
            { t: 10, e: 'grow', p: 2, x: 11, y: 8 },
            { t: 15, e: 'link', a: 1, b: 2 },
            { t: 20, e: 'grow', p: 1, x: 9, y: 9 },
            { t: 30, e: 'season', name: 'Summer' },
            { t: 50, e: 'invade', p: 1, victim: 2, x: 10, y: 8 },
            { t: 60, e: 'unlink', a: 1, b: 2 },
            { t: 80, e: 'died', p: 2, fitness: 42.5 },
        ],
    }],
};

const server = createServer((req, res) => {
    let url = req.url.split('?')[0];
    if (url === '/') url = '/index.html';
    if (url === '/montage.json') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(mockMontage));
        return;
    }

    const filePath = join(webDir, url);
    if (!filePath.startsWith(webDir)) {
        res.writeHead(403);
        res.end('Forbidden');
        return;
    }

    if (existsSync(filePath)) {
        const ext = extname(filePath);
        res.writeHead(200, { 'Content-Type': MIME_TYPES[ext] || 'text/plain' });
        res.end(readFileSync(filePath));
    } else {
        res.writeHead(404);
        res.end('Not Found');
    }
});

server.listen(3333, () => console.log('Test server on http://localhost:3333'));
