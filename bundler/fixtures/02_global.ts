import { hello } from './base.ts';

async function handler(name: string): Promise<string> {
  return await hello(name);
}

export { handler };
