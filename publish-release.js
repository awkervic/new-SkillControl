const https = require('https');
const fs = require('fs');
const path = require('path');
const { spawn } = require('child_process');
const GITHUB_REPO = 'awkervic/new-SkillControl';
const TAG = 'v0.1.3';
const RELEASE_NAME = 'v0.1.3 — 浏览器 UA 伪装 + PROPFIND href 下载 + 403 绝杀';
const RELEASE_BODY = `## v0.1.3

### 修复
- **WebDAV 403 Forbidden 绝杀**：两个客户端函数全部注入 Chrome 120 全套浏览器请求头（User-Agent / Accept / Accept-Language / Accept-Encoding / Cache-Control），伪装请求来源绕过网盘反爬拦截。
- **PROPFIND href 下载**：不再自行构造 URL，改用 PROPFIND 响应中服务器返回的原始 href 路径下载，确保路径绝对一致。
- **冷启动卡死彻底修复**、Time Machine 面板格式修复。
`;
const NSIS = path.join(__dirname, 'src-tauri', 'target', 'release', 'bundle', 'nsis', 'new-SkillControl_0.1.3_x64-setup.exe');
const BIN = path.join(__dirname, 'src-tauri', 'target', 'release', 'new-skillcontrol.exe');

function getToken() {
  return new Promise((resolve, reject) => {
    const cred = spawn('git', ['credential', 'fill'], { cwd: __dirname });
    let out = '';
    cred.stdout.on('data', d => out += d.toString());
    cred.on('close', () => {
      const pwd = out.split('\n').find(l => l.startsWith('password='));
      resolve(pwd ? pwd.split('=')[1] : (process.env.GH_TOKEN || process.env.GITHUB_TOKEN));
    });
    cred.on('error', reject);
    cred.stdin.write('protocol=https\nhost=github.com\n\n');
    cred.stdin.end();
  });
}

function api(method, ep, body, token) {
  return new Promise((resolve, reject) => {
    const url = new URL('https://api.github.com' + ep);
    const opts = { hostname: 'api.github.com', path: url.pathname + url.search, method,
      headers: {'User-Agent':'publish','Accept':'application/vnd.github.v3+json'} };
    if (token) opts.headers['Authorization'] = `token ${token}`;
    if (body) opts.headers['Content-Type'] = 'application/json';
    const req = https.request(opts, res => { let d=''; res.on('data',c=>d+=c); res.on('end',()=>{
      if (res.statusCode<300) try{resolve(JSON.parse(d))}catch{resolve(d)}
      else reject(`API ${method} ${ep} ${res.statusCode}: ${d}`); }); });
    req.on('error', reject);
    if (body) req.write(JSON.stringify(body));
    req.end();
  });
}

function upload(url, file, ctype, token) {
  const stat = fs.statSync(file);
  const u = url.replace('{?name,label}', `?name=${encodeURIComponent(path.basename(file))}`);
  return new Promise((resolve, reject) => {
    const parsed = new URL(u);
    const opts = { hostname: 'uploads.github.com', path: parsed.pathname + parsed.search, method: 'POST',
      headers: {'User-Agent':'publish','Accept':'application/vnd.github.v3+json',
                'Authorization':`token ${token}`,'Content-Type':ctype,'Content-Length':stat.size} };
    const req = https.request(opts, res => { let d=''; res.on('data',c=>d+=c); res.on('end',()=>{
      if (res.statusCode<300) resolve(JSON.parse(d)); else reject(`Upload ${res.statusCode}: ${d}`); }); });
    req.on('error', reject);
    fs.createReadStream(file).pipe(req);
  });
}

async function main() {
  const token = await getToken();
  console.log('[publish] Updating release...');
  let release;
  try { release = await api('GET', `/repos/${GITHUB_REPO}/releases/tags/${TAG}`, null, token);
    for (const a of (release.assets || [])) { await api('DELETE', `/repos/${GITHUB_REPO}/releases/assets/${a.id}`, null, token); }
    release = await api('PATCH', `/repos/${GITHUB_REPO}/releases/${release.id}`, { tag_name: TAG, name: RELEASE_NAME, body: RELEASE_BODY }, token);
  } catch { release = await api('POST', `/repos/${GITHUB_REPO}/releases`, { tag_name: TAG, name: RELEASE_NAME, body: RELEASE_BODY }, token); }

  for (const [file, ct] of [[NSIS,'application/vnd.microsoft.portable-executable'],[BIN,'application/octet-stream']]) {
    if (fs.existsSync(file)) { console.log(`[publish] Uploading ${path.basename(file)}...`);
      try { const r = await upload(release.upload_url, file, ct, token); console.log(`[publish] OK: ${r.name}`); }
      catch(e) { console.error(`[publish] FAIL: ${e}`); }
    } else console.warn(`[publish] Not found: ${file}`);
  }
  console.log(`[publish] ✅ https://github.com/${GITHUB_REPO}/releases/tag/${TAG}`);
}
main().catch(e => { console.error(e); process.exit(1); });
