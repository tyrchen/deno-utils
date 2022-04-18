// deno-fmt-ignore-file
// deno-lint-ignore-file
// This code was bundled using `deno bundle` and it's not recommended to edit it manually
'use strict';

((window) => {
  async function main_module() {
    function delay(ms, options = {}) {
      const { signal } = options;
      if (signal?.aborted) {
        return Promise.reject(
          new DOMException('Delay was aborted.', 'AbortError')
        );
      }
      return new Promise((resolve, reject) => {
        const abort = () => {
          clearTimeout(i);
          reject(new DOMException('Delay was aborted.', 'AbortError'));
        };
        const done = () => {
          signal?.removeEventListener('abort', abort);
          resolve();
        };
        const i = setTimeout(done, ms);
        signal?.addEventListener('abort', abort, { once: true });
      });
    }
    const ERROR_SERVER_CLOSED = 'Server closed';
    const INITIAL_ACCEPT_BACKOFF_DELAY = 5;
    const MAX_ACCEPT_BACKOFF_DELAY = 1e3;
    class Server {
      #port;
      #host;
      #handler;
      #closed = false;
      #listeners = new Set();
      #httpConnections = new Set();
      #onError;
      constructor(serverInit) {
        this.#port = serverInit.port;
        this.#host = serverInit.hostname;
        this.#handler = serverInit.handler;
        this.#onError =
          serverInit.onError ??
          function (error) {
            console.error(error);
            return new Response('Internal Server Error', { status: 500 });
          };
      }
      async serve(listener) {
        if (this.#closed) {
          throw new Deno.errors.Http(ERROR_SERVER_CLOSED);
        }
        this.#trackListener(listener);
        try {
          return await this.#accept(listener);
        } finally {
          this.#untrackListener(listener);
          try {
            listener.close();
          } catch {}
        }
      }
      async listenAndServe() {
        if (this.#closed) {
          throw new Deno.errors.Http(ERROR_SERVER_CLOSED);
        }
        const listener = Deno.listen({
          port: this.#port ?? 80,
          hostname: this.#host ?? '0.0.0.0',
          transport: 'tcp',
        });
        return await this.serve(listener);
      }
      async listenAndServeTls(certFile, keyFile) {
        if (this.#closed) {
          throw new Deno.errors.Http(ERROR_SERVER_CLOSED);
        }
        const listener = Deno.listenTls({
          port: this.#port ?? 443,
          hostname: this.#host ?? '0.0.0.0',
          certFile,
          keyFile,
          transport: 'tcp',
        });
        return await this.serve(listener);
      }
      close() {
        if (this.#closed) {
          throw new Deno.errors.Http(ERROR_SERVER_CLOSED);
        }
        this.#closed = true;
        for (const listener of this.#listeners) {
          try {
            listener.close();
          } catch {}
        }
        this.#listeners.clear();
        for (const httpConn of this.#httpConnections) {
          this.#closeHttpConn(httpConn);
        }
        this.#httpConnections.clear();
      }
      get closed() {
        return this.#closed;
      }
      get addrs() {
        return Array.from(this.#listeners).map((listener) => listener.addr);
      }
      async #respond(requestEvent, httpConn, connInfo) {
        let response;
        try {
          response = await this.#handler(requestEvent.request, connInfo);
        } catch (error) {
          response = await this.#onError(error);
        }
        try {
          await requestEvent.respondWith(response);
        } catch {
          return this.#closeHttpConn(httpConn);
        }
      }
      async #serveHttp(httpConn1, connInfo1) {
        while (!this.#closed) {
          let requestEvent;
          try {
            requestEvent = await httpConn1.nextRequest();
          } catch {
            break;
          }
          if (requestEvent === null) {
            break;
          }
          this.#respond(requestEvent, httpConn1, connInfo1);
        }
        this.#closeHttpConn(httpConn1);
      }
      async #accept(listener) {
        let acceptBackoffDelay;
        while (!this.#closed) {
          let conn;
          try {
            conn = await listener.accept();
          } catch (error) {
            if (
              error instanceof Deno.errors.BadResource ||
              error instanceof Deno.errors.InvalidData ||
              error instanceof Deno.errors.UnexpectedEof ||
              error instanceof Deno.errors.ConnectionReset ||
              error instanceof Deno.errors.NotConnected
            ) {
              if (!acceptBackoffDelay) {
                acceptBackoffDelay = INITIAL_ACCEPT_BACKOFF_DELAY;
              } else {
                acceptBackoffDelay *= 2;
              }
              if (acceptBackoffDelay >= 1e3) {
                acceptBackoffDelay = MAX_ACCEPT_BACKOFF_DELAY;
              }
              await delay(acceptBackoffDelay);
              continue;
            }
            throw error;
          }
          acceptBackoffDelay = undefined;
          let httpConn;
          try {
            httpConn = Deno.serveHttp(conn);
          } catch {
            continue;
          }
          this.#trackHttpConnection(httpConn);
          const connInfo = {
            localAddr: conn.localAddr,
            remoteAddr: conn.remoteAddr,
          };
          this.#serveHttp(httpConn, connInfo);
        }
      }
      #closeHttpConn(httpConn2) {
        this.#untrackHttpConnection(httpConn2);
        try {
          httpConn2.close();
        } catch {}
      }
      #trackListener(listener1) {
        this.#listeners.add(listener1);
      }
      #untrackListener(listener2) {
        this.#listeners.delete(listener2);
      }
      #trackHttpConnection(httpConn3) {
        this.#httpConnections.add(httpConn3);
      }
      #untrackHttpConnection(httpConn4) {
        this.#httpConnections.delete(httpConn4);
      }
    }
    async function serve(handler1, options = {}) {
      const server = new Server({
        port: options.port ?? 8e3,
        hostname: options.hostname ?? '0.0.0.0',
        handler: handler1,
        onError: options.onError,
      });
      if (options?.signal) {
        options.signal.onabort = () => server.close();
      }
      return await server.listenAndServe();
    }
    const randomInteger = (minimum, maximum) =>
      Math.floor(Math.random() * (maximum - minimum + 1) + minimum);
    const createAbortError = () => {
      const error = new Error('Delay aborted');
      error.name = 'AbortError';
      return error;
    };
    const createDelay =
      ({ clearTimout: defaultClear, setTimeout: set, willResolve }) =>
      (ms, options) => {
        if (options?.signal && options.signal.aborted) {
          return Promise.reject(createAbortError());
        }
        let timeoutId;
        let settle;
        let rejectFn;
        const clear = defaultClear || clearTimeout;
        const signalListener = () => {
          clear(timeoutId);
          rejectFn(createAbortError());
        };
        const cleanup = () => {
          if (options?.signal) {
            options?.signal.removeEventListener('abort', signalListener);
          }
        };
        let delayPromise;
        delayPromise = new Promise((resolve, reject) => {
          settle = () => {
            cleanup();
            if (willResolve) {
              resolve(options?.value);
            } else {
              reject(options?.value);
            }
          };
          rejectFn = reject;
          timeoutId = (set || setTimeout)(settle, ms);
        });
        if (options?.signal) {
          options?.signal.addEventListener('abort', signalListener, {
            once: true,
          });
        }
        delayPromise.clear = () => {
          clear(timeoutId);
          timeoutId = null;
          settle();
        };
        return delayPromise;
      };
    let delay1;
    delay1 = createDelay({ willResolve: true });
    delay1.reject = createDelay({ willResolve: false });
    delay1.range = (minimum, maximum, options) =>
      delay1(randomInteger(minimum, maximum), options);
    delay1.createWithTimers = ({ clearTimeout, setTimeout }) => {
      delay1 = createDelay({ clearTimeout, setTimeout, willResolve: true });
      delay1.reject = createDelay({
        clearTimeout,
        setTimeout,
        willResolve: false,
      });
      return delay1;
    };
    async function handler(req) {
      await delay1(100);
      const body = `!! Your user-agent is: ${
        req.headers.get('user-agent') ?? 'Unknown'
      }`;
      return new Response(body, { status: 200 });
    }
    await serve(handler, { port: 8080 });
  }

  window.main_module = main_module;
})(globalThis);
