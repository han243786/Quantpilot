import { readFileSync, appendFileSync, existsSync, mkdirSync } from 'fs';

const QS = readFileSync('tests/scenarios/testnet_spot.qs', 'utf-8');
const LOG = 'markdown/测试/testnet_长时间测试.log';
const INTERVAL = 60000; // 1 minute between runs
const DURATION_MIN = 60; // 60 minutes
const TOTAL = DURATION_MIN;
let run = 0, totalOrders = 0, totalErrors = 0, totalLatency = 0;

if (!existsSync('markdown/测试')) mkdirSync('markdown/测试', { recursive: true });

function log(msg) {
  const ts = new Date().toISOString().slice(11, 19);
  const line = `[${ts}] ${msg}`;
  console.log(line);
  appendFileSync(LOG, line + '\n');
}

async function execute() {
  run++;
  const t0 = Date.now();

  try {
    const r = await fetch('http://127.0.0.1:3000/api/test/scenario/run', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ source: QS }),
    });
    const j = await r.json();
    const latency = Date.now() - t0;
    totalLatency += latency;

    const step = j.steps[0];
    if (step?.status === 'passed') {
      totalOrders++;
      const msg = step.message || '';
      const orderId = (msg.match(/order (\d+)/) || [])[1] || '?';
      const equity = (msg.match(/equity=([\d.]+)/) || [])[1] || '?';
      log(
        `#${run}/${TOTAL} ✅ order=${orderId.slice(-8)} equity=${equity} latency=${latency}ms`
      );
    } else {
      totalErrors++;
      log(`#${run}/${TOTAL} ❌ ${step?.status} ${step?.message?.slice(0, 120)}`);
    }
  } catch (e) {
    totalErrors++;
    log(`#${run}/${TOTAL} ❌ CRASH: ${e.message?.slice(0, 100)}`);
  }

  if (run >= TOTAL) {
    const avgLatency = (totalLatency / TOTAL).toFixed(0);
    log('\n═══════════════════════════');
    log(`测试完成: ${TOTAL} 轮, ${DURATION_MIN} 分钟`);
    log(`成功下单: ${totalOrders}/${TOTAL}`);
    log(`错误: ${totalErrors}`);
    log(`平均延迟: ${avgLatency}ms`);
    log(`成功率: ${((totalOrders / TOTAL) * 100).toFixed(1)}%`);
    log('═══════════════════════════');
    process.exit(totalErrors > 0 ? 1 : 0);
  } else {
    setTimeout(execute, INTERVAL);
  }
}

log(`═══ 模拟盘长时间测试开始 ═══`);
log(`间隔: ${INTERVAL / 1000}s | 时长: ${DURATION_MIN}min | 轮次: ${TOTAL}`);
log(`策略: SMA(10,30)交叉 → BUY 0.001 BTC | @run testnet`);
log('');
execute();
