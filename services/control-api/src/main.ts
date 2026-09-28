import { createApp } from './app';
const app = createApp();
await app.listen({ host: '127.0.0.1', port: 4318 });
for (const signal of ['SIGINT', 'SIGTERM'] as const) process.once(signal, () => { void app.close(); });
console.log('Conductor scaffold API: http://127.0.0.1:4318/healthz');
