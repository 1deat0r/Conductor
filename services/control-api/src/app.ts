import Fastify from 'fastify';
import { scaffoldHealth } from '@conductor/protocol';
export function createApp() {
  const app = Fastify({ logger: false, bodyLimit: 16_384 });
  app.get('/healthz', async () => scaffoldHealth('control-api'));
  app.post('/v1/commands', async (_request, reply) => reply.code(501).send({error: 'not_implemented', message: 'Execution is unavailable in the S0 scaffold.'}));
  return app;
}
