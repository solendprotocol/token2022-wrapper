const path = require('path');
const { Solita } = require('@metaplex-foundation/solita');
const {
  rustbinMatch,
  confirmAutoMessageConsole,
} = require('@metaplex-foundation/rustbin')
const { spawn } = require('child_process');

const programDir = path.join(__dirname, "..");
const cargoToml = path.join(programDir, "Cargo.toml");
const generatedIdlDir = __dirname;
const generatedSDKDir = path.join(__dirname, "src", "generated");
const rootDir = path.join(__dirname, ".crates");

const PROGRAM_NAME = 'token2022_wrapper';
const rustbinConfig = {
  rootDir,
  binaryName: 'shank',
  binaryCrateName: 'shank-cli',
  libName: 'shank',
  dryRun: false,
  cargoToml,
}

async function main() {
  const { fullPathToBinary: shankExecutable } = await rustbinMatch(
    rustbinConfig,
    confirmAutoMessageConsole
  )
  const shank = spawn(shankExecutable, ['idl', '--out-dir', generatedIdlDir, '--crate-root', programDir])
    .on('error', (err) => {
      console.error(err);
      if (err.code === 'ENOENT') {
        console.error(
          'Ensure that `shank` is installed and in your path, see:\n  https://github.com/metaplex-foundation/shank\n',
        );
      }
      process.exit(1);
    })
    .on('exit', () => {
      generateTypeScriptSDK();
    });

  shank.stdout.on('data', (buf) => console.log(buf.toString('utf8')));
  shank.stderr.on('data', (buf) => console.error(buf.toString('utf8')));
}

async function generateTypeScriptSDK() {
  console.error('Generating TypeScript SDK to %s', generatedSDKDir);
  const generatedIdlPath = path.join(generatedIdlDir, `${PROGRAM_NAME}.json`);

  const idl = require(generatedIdlPath);
  const gen = new Solita(idl, { formatCode: true });
  await gen.renderAndWriteTo(generatedSDKDir);

  console.error('Success!');

  process.exit(0);
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})