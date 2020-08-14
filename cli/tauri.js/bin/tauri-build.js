const parseArgs = require('minimist');

const argv = parseArgs(process.argv.slice(2), {
  alias: {
    h: 'help',
    d: 'debug',
    t: 'target',
    v: 'verbose'
  },
  boolean: ['h', 'd', 'v']
});

if (argv.help) {
  console.log(`
  Description
    Tauri build.
  Usage
    $ tauri build
  Options
    --help, -h     Displays this message
    --debug, -d    Builds with the debug flag
    --target, -t   Comma-separated list of target triples to build against
    --verbose, -v  Enable verbose logging
  `);
  process.exit(0);
}

async function run() {
  const build = require('../dist/api/build');

  await build({
    ctx: {
      debug: argv.debug,
      target: argv.target
    },
    verbose: argv.verbose
  }).promise;
}

run();
