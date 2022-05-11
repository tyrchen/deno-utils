import { serve } from 'https://deno.land/std@0.134.0/http/server.ts';
import { delay } from 'https://deno.land/x/delay@v0.2.0/mod.ts';

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
