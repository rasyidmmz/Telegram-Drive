import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

function decodeEnvelope(encodedEnvelope) {
  const value = encodedEnvelope.trim();
  if (value.startsWith('untrusted comment:')) {
    return value;
  }

  return Buffer.from(value, 'base64').toString('utf8');
}

export function keyIdFromEncodedEnvelope(encodedEnvelope) {
  const payloadLine = decodeEnvelope(encodedEnvelope)
    .split(/\r?\n/)
    .find((line) => line && !line.endsWith('comment:') && !line.includes('comment:'));

  if (!payloadLine) {
    throw new Error('Updater key envelope does not contain a key payload');
  }

  const payload = Buffer.from(payloadLine, 'base64');
  if (payload.length < 10) {
    throw new Error('Updater key payload is too short');
  }

  return Buffer.from(payload.subarray(2, 10)).reverse().toString('hex').toUpperCase();
}

export function assertMatchingKeyIds(publicKey, signature) {
  const publicKeyId = keyIdFromEncodedEnvelope(publicKey);
  const signatureKeyId = keyIdFromEncodedEnvelope(signature);

  if (publicKeyId !== signatureKeyId) {
    throw new Error(
      `Updater signing key mismatch: configured ${publicKeyId}, signature ${signatureKeyId}`,
    );
  }

  return publicKeyId;
}

export function assertWindowsUpgradeConfig(config) {
  if (config.productName !== 'Teledrive') {
    throw new Error('Windows installer productName must remain Teledrive');
  }
  if (config.identifier !== 'com.rasyidmmz.teledrive') {
    throw new Error(
      'Windows installer identifier must remain com.rasyidmmz.teledrive',
    );
  }
  if (!config.bundle?.targets?.includes('nsis')) {
    throw new Error('Windows installer must continue to use the NSIS target');
  }
  if (config.bundle?.windows?.nsis?.installMode !== 'currentUser') {
    throw new Error('Windows NSIS installMode must remain currentUser');
  }
  if (config.plugins?.updater?.windows?.installMode !== 'passive') {
    throw new Error('Windows updater installMode must remain passive');
  }
}

function verifyConfiguredSigningKey() {
  const appDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
  const configPath = path.join(appDir, 'src-tauri', 'tauri.conf.json');
  const privateKey = process.env.TAURI_SIGNING_PRIVATE_KEY;

  if (!privateKey) {
    throw new Error('TAURI_SIGNING_PRIVATE_KEY is not set');
  }

  const config = JSON.parse(fs.readFileSync(configPath, 'utf8'));
  assertWindowsUpgradeConfig(config);
  const publicKey = config.plugins?.updater?.pubkey;
  if (!publicKey) {
    throw new Error('plugins.updater.pubkey is not configured');
  }

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'teledrive-updater-key-'));
  const probePath = path.join(tempDir, 'probe.txt');
  const signaturePath = `${probePath}.sig`;
  const signerEnv = {
    ...process.env,
    TAURI_PRIVATE_KEY: privateKey,
  };
  const privateKeyPassword = process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD;
  if (privateKeyPassword) {
    signerEnv.TAURI_PRIVATE_KEY_PASSWORD = privateKeyPassword;
  } else {
    delete signerEnv.TAURI_PRIVATE_KEY_PASSWORD;
  }

  try {
    fs.writeFileSync(probePath, 'teledrive updater signing key probe');
    execFileSync(
      process.execPath,
      [
        path.join(appDir, 'node_modules', '@tauri-apps', 'cli', 'tauri.js'),
        'signer',
        'sign',
        probePath,
      ],
      {
        cwd: appDir,
        env: signerEnv,
        stdio: ['ignore', 'ignore', 'inherit'],
      },
    );

    if (!fs.existsSync(signaturePath)) {
      throw new Error(`Tauri signer did not create ${signaturePath}`);
    }

    const keyId = assertMatchingKeyIds(
      publicKey,
      fs.readFileSync(signaturePath, 'utf8'),
    );
    console.log(`Updater signing key verified: ${keyId}`);
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  verifyConfiguredSigningKey();
}
