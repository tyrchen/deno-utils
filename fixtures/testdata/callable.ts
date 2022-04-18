function process(data: string): string {
  return data;
}

async function async_process(url: string): Promise<number> {
  const data = await fetch(url);
  return data.status;
}

globalThis.process = process;
globalThis.async_process = async_process;
