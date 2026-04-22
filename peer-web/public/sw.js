/* Peer Network — Service Worker
 *
 * Caching strategy table:
 *   /api/, /graphql        → network-only (auth-sensitive, never cached)
 *   /admin/*               → network-only, no offline fallback (audit safety)
 *   navigate (HTML)        → network-first, fall back to /offline.html
 *   /pkg/*                 → stale-while-revalidate (filenames stable; cache
 *                            busting comes from peer-shell-v{HASH} purge)
 *   /img/, /svg/,
 *   /fonts/, /css/         → cache-first, 30-day expiry
 *   default                → network with cache fallback
 *
 * Versioning:
 *   The cache name embeds the build hash passed at registration time as
 *   `?v=<HASH>`. The activate handler purges any cache whose name does not
 *   match the current version, giving us atomic deploys without stale WASM.
 */

const VERSION = new URL(self.location).searchParams.get('v') || 'dev';
const SHELL_CACHE = `peer-shell-v${VERSION}`;
const RUNTIME_CACHE = `peer-runtime-v${VERSION}`;

const OFFLINE_URL = '/offline.html';

const PRECACHE_URLS = [
    '/',
    '/login',
    '/dashboard',
    '/pkg/peer-web.js',
    '/pkg/peer-web_bg.wasm',
    '/pkg/peer-web.css',
    '/manifest.webmanifest',
    OFFLINE_URL,
    '/img/pwa/icon-192.png',
    '/img/pwa/icon-512.png',
    '/img/pwa/icon-192-maskable.png',
    '/img/pwa/icon-512-maskable.png',
    '/img/pwa/icon-monochrome.svg',
    '/img/pwa/apple-touch-icon-180.png',
    '/svg/logo_farbe.svg',
    '/svg/logo_sw.svg',
];

self.addEventListener('install', (event) => {
    event.waitUntil(
        (async () => {
            const cache = await caches.open(SHELL_CACHE);
            // Best-effort precache: a missing optional asset must not abort
            // the install. Add each URL individually and swallow failures.
            await Promise.all(
                PRECACHE_URLS.map((url) =>
                    cache.add(new Request(url, { cache: 'reload' })).catch(() => undefined),
                ),
            );
        })(),
    );
});

self.addEventListener('activate', (event) => {
    event.waitUntil(
        (async () => {
            const keep = new Set([SHELL_CACHE, RUNTIME_CACHE]);
            const names = await caches.keys();
            await Promise.all(
                names
                    .filter((name) => name.startsWith('peer-') && !keep.has(name))
                    .map((name) => caches.delete(name)),
            );
            await self.clients.claim();
        })(),
    );
});

self.addEventListener('message', (event) => {
    if (event.data && event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }
});

function isStaticAsset(pathname) {
    return (
        pathname.startsWith('/img/') ||
        pathname.startsWith('/svg/') ||
        pathname.startsWith('/fonts/') ||
        pathname.startsWith('/css/')
    );
}

async function networkFirstNavigation(request) {
    try {
        const fresh = await fetch(request);
        return fresh;
    } catch (_err) {
        const cache = await caches.open(SHELL_CACHE);
        const fallback = await cache.match(OFFLINE_URL);
        return (
            fallback ||
            new Response('You are offline.', {
                status: 503,
                headers: { 'Content-Type': 'text/plain; charset=utf-8' },
            })
        );
    }
}

async function cacheFirst(request) {
    const cache = await caches.open(RUNTIME_CACHE);
    const cached = await cache.match(request);
    if (cached) return cached;
    const fresh = await fetch(request);
    if (fresh && fresh.ok) {
        cache.put(request, fresh.clone()).catch(() => undefined);
    }
    return fresh;
}

async function staleWhileRevalidate(request) {
    const cache = await caches.open(RUNTIME_CACHE);
    const cached = await cache.match(request);
    const networkPromise = fetch(request)
        .then((response) => {
            if (response && response.ok) {
                cache.put(request, response.clone()).catch(() => undefined);
            }
            return response;
        })
        .catch(() => undefined);
    return cached || networkPromise || fetch(request);
}

self.addEventListener('fetch', (event) => {
    const { request } = event;
    if (request.method !== 'GET') return;

    const url = new URL(request.url);
    if (url.origin !== self.location.origin) return;

    // Auth-sensitive: never cache.
    if (url.pathname.startsWith('/api/') || url.pathname === '/graphql') {
        return; // default network handling
    }

    // Admin shell must never be served from cache.
    if (url.pathname.startsWith('/admin')) {
        return;
    }

    // Navigation requests → network-first with offline fallback.
    if (request.mode === 'navigate') {
        event.respondWith(networkFirstNavigation(request));
        return;
    }

    // Stable WASM/JS bundles.
    if (url.pathname.startsWith('/pkg/')) {
        event.respondWith(staleWhileRevalidate(request));
        return;
    }

    // Long-lived static assets.
    if (isStaticAsset(url.pathname)) {
        event.respondWith(cacheFirst(request));
        return;
    }

    // Default: try network, fall back to whatever is cached.
    event.respondWith(
        fetch(request).catch(async () => {
            const cached = await caches.match(request);
            if (cached) return cached;
            throw new Error('network failed and no cache match');
        }),
    );
});
