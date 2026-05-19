// Ed25519 鍵生成・署名・検証。@noble/curves を使用しオフライン署名を可能にする
import { ed25519 } from '@noble/curves/ed25519';
import { bytesToHex, hexToBytes, randomBytes } from '@noble/hashes/utils';

export interface Ed25519KeyPair {
  privateKeyHex: string;
  publicKeyHex: string;
}

export function generateKeyPair(): Ed25519KeyPair {
  const privateKey = randomBytes(32);
  const publicKey = ed25519.getPublicKey(privateKey);
  return {
    privateKeyHex: bytesToHex(privateKey),
    publicKeyHex: bytesToHex(publicKey),
  };
}

export function sign(messageBytes: Uint8Array, privateKeyHex: string): string {
  const signature = ed25519.sign(messageBytes, hexToBytes(privateKeyHex));
  return bytesToHex(signature);
}

export function verify(signatureHex: string, messageBytes: Uint8Array, publicKeyHex: string): boolean {
  return ed25519.verify(hexToBytes(signatureHex), messageBytes, hexToBytes(publicKeyHex));
}
