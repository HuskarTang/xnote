import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const pkgPath = path.resolve(__dirname, '../package.json');
const tauriPath = path.resolve(__dirname, '../src-tauri/tauri.conf.json');

try {
  // 读取 package.json
  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf-8'));
  const version = pkg.version;

  // 读取并更新 tauri.conf.json
  const tauriConfig = JSON.parse(fs.readFileSync(tauriPath, 'utf-8'));
  tauriConfig.package.version = version;

  fs.writeFileSync(tauriPath, JSON.stringify(tauriConfig, null, 2) + '\n');

  console.log(`✅ Synced version ${version} to tauri.conf.json`);
} catch (error) {
  console.error('❌ Failed to sync version:', error.message);
  process.exit(1);
}
