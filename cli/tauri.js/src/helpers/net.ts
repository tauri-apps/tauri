// forked from https://github.com/quasarframework/quasar/blob/dev/app/lib/helpers/net.js

import net from 'net';

async function findClosestOpenPort(
  port: number,
  host: string
): Promise<number> {
  return await isPortAvailable(port, host).then((isAvailable) => {
    if (isAvailable) {
      return port;
    } else if (port < 65535) {
      return findClosestOpenPort(port + 1, host);
    } else {
      throw new Error('ERROR_NETWORK_PORT_NOT_AVAIL');
    }
  });
}

async function isPortAvailable(port: number, host: string): Promise<boolean> {
  return await new Promise((resolve, reject) => {
    const tester = net
      .createServer()
      .once('error', (err: NodeJS.ErrnoException) => {
        if (err.code === 'EADDRNOTAVAIL') {
          reject(new Error('ERROR_NETWORK_ADDRESS_NOT_AVAIL'));
        } else if (err.code === 'EADDRINUSE') {
          resolve(false); // host/port in use
        } else {
          reject(err);
        }
      })
      .once('listening', () => {
        tester
          .once('close', () => {
            resolve(true); // found available host/port
          })
          .close();
      })
      .on('error', (err: any) => {
        reject(err);
      })
      .listen(port, host);
  });
}

export { findClosestOpenPort, isPortAvailable };
