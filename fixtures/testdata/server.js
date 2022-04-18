const server = Deno.listen({ port: 8080 });
// console.log(`HTTP webserver running. Access it at: http://localhost:8080/`);

for await (const conn of server) {
  // console.log(`Connection from ${JSON.stringify(conn.remoteAddr)}`);
  serveHttp(conn);
}

async function serveHttp(conn) {
  const httpConn = Deno.serveHttp(conn);
  for await (const requestEvent of httpConn) {
    const body = `Your user-agent is:\n\n${
      requestEvent.request.headers.get('user-agent') ?? 'Unknown'
    }`;

    await requestEvent.respondWith(
      new Response(body, {
        status: 200,
      })
    );
  }
}
