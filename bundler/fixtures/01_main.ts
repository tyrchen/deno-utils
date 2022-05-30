import { serve } from 'https://cdn.jsdelivr.net/gh/denoland/deno_std@main/http/server.ts';
import { delay } from 'https://cdn.jsdelivr.net/gh/deno-delay/delay@main/src/delay.ts';

async function handler(req: Request): Promise<Response> {
  await delay(100);
  const body = `Your user-agent is: ${
    req.headers.get('user-agent') ?? 'Unknown'
  }`;
  return new Response(body, {
    status: 200,
  });
}

await serve(handler, { port: 8080 });
