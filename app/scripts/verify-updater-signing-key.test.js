import assert from 'node:assert/strict';
import test from 'node:test';

import {
  assertMatchingKeyIds,
  assertWindowsUpgradeConfig,
  keyIdFromEncodedEnvelope,
} from './verify-updater-signing-key.js';

function encodedEnvelope(algorithm, keyIdHex) {
  const payload = Buffer.concat([
    Buffer.from(algorithm, 'ascii'),
    Buffer.from(keyIdHex, 'hex').reverse(),
    Buffer.alloc(64),
  ]).toString('base64');
  const text = `untrusted comment: test\n${payload}\n`;
  return Buffer.from(text).toString('base64');
}

test('extracts the same key ID from public key and signature envelopes', () => {
  const keyId = 'CB120529ED019813';

  assert.equal(keyIdFromEncodedEnvelope(encodedEnvelope('Ed', keyId)), keyId);
  assert.equal(keyIdFromEncodedEnvelope(encodedEnvelope('ED', keyId)), keyId);
});

test('rejects a signature created by a different key', () => {
  const publicKey = encodedEnvelope('Ed', '507B700E3497963C');
  const signature = encodedEnvelope('ED', 'CB120529ED019813');

  assert.throws(
    () => assertMatchingKeyIds(publicKey, signature),
    /Updater signing key mismatch/,
  );
});

test('accepts a signature created by the configured key', () => {
  const publicKey = encodedEnvelope('Ed', 'CB120529ED019813');
  const signature = encodedEnvelope('ED', 'CB120529ED019813');

  assert.doesNotThrow(() => assertMatchingKeyIds(publicKey, signature));
});

test('requires a stable current-user NSIS upgrade identity', () => {
  const config = {
    productName: 'Teledrive',
    identifier: 'com.rasyidmmz.teledrive',
    bundle: {
      targets: ['nsis'],
      windows: { nsis: { installMode: 'currentUser' } },
    },
    plugins: {
      updater: { windows: { installMode: 'passive' } },
    },
  };

  assert.doesNotThrow(() => assertWindowsUpgradeConfig(config));
  assert.throws(
    () => assertWindowsUpgradeConfig({ ...config, identifier: 'com.example.changed' }),
    /installer identifier must remain/,
  );
});
