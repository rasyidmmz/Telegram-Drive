import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import crypto from 'crypto';
import https from 'https';
import { execSync } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const whisperDir = path.join(__dirname, 'src-tauri', 'resources', 'whisper');

const ZIP_URL = 'https://github.com/ggml-org/whisper.cpp/releases/download/v1.9.1/whisper-bin-x64.zip';
const ZIP_HASH = '7d8be46ecd31828e1eb7a2ecdd0d6b314feafd82163038ab6092594b0a063539';
const MODEL_URL = 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin';
const MODEL_HASH = 'a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002';

function sha256(filePath) {
    const fileBuffer = fs.readFileSync(filePath);
    const hashSum = crypto.createHash('sha256');
    hashSum.update(fileBuffer);
    return hashSum.digest('hex');
}

function download(url, dest) {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(dest);
        https.get(url, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                https.get(response.headers.location, (res) => {
                    res.pipe(file);
                    file.on('finish', () => file.close(resolve));
                }).on('error', reject);
            } else {
                response.pipe(file);
                file.on('finish', () => file.close(resolve));
            }
        }).on('error', reject);
    });
}

async function run() {
    fs.mkdirSync(whisperDir, { recursive: true });
    const zipPath = path.join(whisperDir, 'whisper-bin.zip');
    const modelPath = path.join(whisperDir, 'ggml-base.en.bin');

    console.log('Downloading Whisper CLI...');
    await download(ZIP_URL, zipPath);
    console.log('Verifying Whisper CLI hash...');
    const zipActualHash = sha256(zipPath);
    if (zipActualHash !== ZIP_HASH) {
        throw new Error(`Whisper ZIP hash mismatch. Expected: ${ZIP_HASH}, Got: ${zipActualHash}`);
    }

    console.log('Downloading Whisper model...');
    await download(MODEL_URL, modelPath);
    console.log('Verifying Whisper model hash...');
    const modelActualHash = sha256(modelPath);
    if (modelActualHash !== MODEL_HASH) {
        throw new Error(`Whisper Model hash mismatch. Expected: ${MODEL_HASH}, Got: ${modelActualHash}`);
    }

    console.log('Extracting Whisper CLI...');
    execSync(`tar -xf "${zipPath}" -C "${whisperDir}"`);
    fs.unlinkSync(zipPath);

    // Move files from Release/ to whisper/
    const releaseDir = path.join(whisperDir, 'Release');
    if (fs.existsSync(releaseDir)) {
        const files = fs.readdirSync(releaseDir);
        for (const file of files) {
            fs.renameSync(path.join(releaseDir, file), path.join(whisperDir, file));
        }
        fs.rmdirSync(releaseDir);
    }
    console.log('Whisper resources setup successfully.');
}

run().catch(err => {
    console.error(err);
    process.exit(1);
});
