const fs = require("fs");

function makeBacktest(label, initialCapital, drift, vol, tradesPerMonth) {
  const points = [];
  const benchPoints = [];
  let eq = initialCapital, bench = initialCapital, peak = eq, maxDD = 0;
  let seed = label.charCodeAt(0)*137 + label.charCodeAt(1)*31;
  const rand = () => { seed=(seed*1664525+1013904223)>>>0; return (seed>>>0)/4294967296; };
  const norm = () => { let s=0; for(let i=0;i<12;i++) s+=rand(); return (s-6)/1.0; };

  for (let i=0; i<300; i++) {
    const r = norm();
    let dailyReturn;
    if (rand()<0.75) dailyReturn = drift + vol*r;
    else if (rand()<0.5) dailyReturn = drift*3 + vol*2*r;
    else dailyReturn = -drift*5 + vol*3*r;
    eq *= (1+dailyReturn);
    bench *= (1+drift*0.5+vol*0.3*r);
    if (eq>peak) peak=eq;
    const dd=(peak-eq)/peak; if (dd>maxDD) maxDD=dd;
    points.push({ts_ms:1730000000000+i*86400000,equity:eq,cash_balance:eq*0.2,net_notional:eq*0.8});
    benchPoints.push({ts_ms:1730000000000+i*86400000,equity:bench,cash_balance:bench,net_notional:0});
  }

  const totalReturn = (eq-initialCapital)/initialCapital;
  const dr = [];
  for (let i=1; i<points.length; i++) dr.push((points[i].equity-points[i-1].equity)/points[i-1].equity);
  const avgDr = dr.reduce((a,b)=>a+b,0)/dr.length;
  const stdDr = Math.sqrt(dr.reduce((s,v)=>s+Math.pow(v-avgDr,2),0)/(dr.length-1));
  const annVol = stdDr*Math.sqrt(252);
  const annRet = Math.pow(1+totalReturn,365/300)-1;
  const sharpe = annVol>0 ? annRet/annVol : 0;
  const downDr = dr.filter(r=>r<0);
  const downVol = downDr.length>0 ? Math.sqrt(downDr.reduce((s,v)=>s+v*v,0)/downDr.length)*Math.sqrt(252) : 0;
  const sortino = downVol>0 ? annRet/downVol : 0;
  const calmar = maxDD>0 ? annRet/maxDD : 0;

  // Monthly returns
  const monthData = [];
  let monthStart = 0, monthStartIdx = 0;
  for (let i=1; i<points.length; i++) {
    const d = new Date(points[i].ts_ms);
    const m = d.getUTCFullYear()*12 + d.getUTCMonth();
    if (i===1) { monthStart = m; monthStartIdx = 0; }
    if (m !== monthStart || i === points.length-1) {
      monthData.push({
        period: d.getUTCFullYear()+"-"+String(d.getUTCMonth()+1).padStart(2,"0"),
        return_ratio: points[monthStartIdx] ? (points[i].equity-points[monthStartIdx].equity)/points[monthStartIdx].equity : 0,
        trade_count: Math.floor(rand()*10+5)
      });
      monthStart = m; monthStartIdx = i;
    }
  }

  // VaR/CVaR
  const sorted = [...dr].sort((a,b)=>a-b);
  const varIdx = Math.floor(sorted.length*0.05);
  const var95 = sorted[varIdx] || 0;
  const cvar95 = sorted.slice(0,varIdx+1).reduce((a,b)=>a+b,0)/(varIdx+1);

  // Streaks
  let currW=0, currL=0, maxW=0, maxL=0;
  for (const r of dr) { if(r>0){currW++;currL=0;if(currW>maxW)maxW=currW;} else if(r<0){currL++;currW=0;if(currL>maxL)maxL=currL;} }

  const tradeCount = Math.floor(300*(tradesPerMonth/30)*(1+vol*50));
  const winCount = Math.floor(tradeCount*0.55);
  const lossCount = tradeCount - winCount;
  const emptyDigest = {algorithm:"sha256_canonical_json",value:""};

  const summary = {
    step_count:300, trade_count:tradeCount, total_return_ratio:totalReturn, final_equity:eq,
    net_profit:eq-initialCapital, win_rate:winCount/tradeCount,
    annualized_return:annRet, annualized_volatility:annVol,
    risk_adjusted:{sharpe_ratio:sharpe,sortino_ratio:sortino,calmar_ratio:calmar,var_95:var95,cvar_95:cvar95},
    trade_analysis:{profit_factor:maxL>0?(dr.filter(r=>r>0).reduce((a,b)=>a+b,0)/Math.abs(dr.filter(r=>r<0).reduce((a,b)=>a+b,0))):2,avg_win:dr.filter(r=>r>0).reduce((a,b)=>a+b,0)/Math.max(1,dr.filter(r=>r>0).length),avg_loss:Math.abs(dr.filter(r=>r<0).reduce((a,b)=>a+b,0))/Math.max(1,dr.filter(r=>r<0).length),max_consecutive_wins:maxW,max_consecutive_losses:maxL},
    drawdown_analysis:{max_drawdown_ratio:maxDD,max_drawdown_duration_days:Math.floor(maxDD*500),avg_drawdown_duration_days:Math.floor(maxDD*200)},
    benchmark_comparison:{benchmark_total_return:(bench-initialCapital)/initialCapital,alpha:annRet-(bench-initialCapital)/initialCapital,beta:1.2,information_ratio:sharpe*0.8},
    skewness:dr.reduce((s,v)=>s+Math.pow((v-avgDr)/stdDr,3),0)*dr.length/((dr.length-1)*(dr.length-2)),
    kurtosis:dr.reduce((s,v)=>s+Math.pow((v-avgDr)/stdDr,4),0)*dr.length/((dr.length-1)*(dr.length-2))-3
  };

  const exec = {initial_cash_balance:initialCapital,taker_fee_bps:label.includes("aggressive")?5:15,default_slippage_bps:label.includes("aggressive")?3:8,total_cost_buffer_bps:label.includes("aggressive")?10:25,time_in_force:"Gtc",allow_partial_fills:false,latency_assumption_ms:null};
  const manifest = {schema_version:"quantpilot/reproducibility-manifest/v1",manifest_id:"manifest_"+label,backtest_id:label,graph_id:"graph_"+label,compile_id:"compile_"+label,created_at_ms:1730000000000,protocol_name:"Strategy "+label,config_hash:"hash_"+label,account:{equity_estimate:eq,cash_balance:eq*0.2,available_cash_balance:eq*0.18,frozen_cash_balance:eq*0.02,total_leverage:0,total_gross_notional:0,total_net_notional:0,positions:1,open_order_count:0,open_orders:[]},summary:summary,backtest_spec:{schema_version:"quantpilot/backtest-spec/v1",backtest_id:label,replay_source:"deterministic_mock",requested_at_ms:1730000000000,run_spec:{schema_version:"quantpilot/run-spec/v1",run_mode:"backtest",graph_id:"graph_"+label,compile_id:"compile_"+label,runtime_mode:"backtest",protocol_name:"Strategy "+label,config_hash:"hash_"+label,datasets:[],execution_assumptions:exec,core_ir_digest:emptyDigest},market_data_snapshot:{snapshot_id:"snap_"+label,replay_source:"deterministic_mock",captured_at_ms:1730000000000,datasets:[],quotes:[],klines:[]}},compile_artifacts:null,governance:{schema_version:"quantpilot/runtime-governance/v1",governance_source:"legacy_default",capability_hash:"sha256:screenshot_test",strategy_version:"v1",parameter_version:"v1",deployment_revision:"v1",capability_api_version:"quantpilot/capabilities/v1",runtime_support_boundary:{runtime_modes:["paper"],execution_module_keys:["builtin.execution.paper"]},indicator_kinds:["ma_cross","rsi","macd","momentum","spread","z_score"],attested_at_ms:1730000000000,attestation_signature:"",permission_boundary:{model_version:"quantpilot/permission-boundary/v1",execution_owner_module:"builtin.execution.paper",live_execution_allowed:false,ai_write_policy:"proposal_only",plugin_network_default:"deny",non_execution_order_access:"deny"}},output_artifacts:[],backtest_output_digest:emptyDigest};

  const dir = "storage/backtests/"+label;
  fs.mkdirSync(dir,{recursive:true});
  fs.writeFileSync(dir+"/manifest.json", JSON.stringify(manifest,null,2));
  fs.writeFileSync(dir+"/backtest_output.json", JSON.stringify({mode:"historical_replay",started_at_ms:1730000000000,ended_at_ms:1730000000000+300*86400000,sessions:[],equity_curve:points,benchmark_equity_curve:benchPoints,period_returns:monthData,summary:summary,final_portfolio:{cash_balance:eq*0.2,available_cash_balance:eq*0.18,frozen_cash_balance:eq*0.02,total_leverage:0,total_gross_notional:0,total_net_notional:0,positions:[],open_orders:[],open_order_count:0},debug_values:null},null,2));
  fs.writeFileSync(dir+"/event_log.json", JSON.stringify({schema_version:"v1",artifact_id:"",backtest_id:label,digest:emptyDigest,event_count:tradeCount*5,events:[]}));
  fs.writeFileSync(dir+"/trade_ledger.json", JSON.stringify({schema_version:"v1",artifact_id:"",backtest_id:label,digest:emptyDigest,trades:[],trade_count:tradeCount,summary:null}));
  fs.writeFileSync(dir+"/equity_curve.json", JSON.stringify({schema_version:"v1",artifact_id:"",backtest_id:label,digest:emptyDigest,points:points,point_count:points.length}));
  fs.writeFileSync(dir+"/metrics.json", JSON.stringify({schema_version:"v1",artifact_id:"",backtest_id:label,digest:emptyDigest,summary:summary,event_count:tradeCount*5,session_count:300,started_at_ms:1730000000000,ended_at_ms:1730000000000+300*86400000,final_account:{cash_balance:eq*0.2,available_cash_balance:eq*0.18,frozen_cash_balance:eq*0.02,total_leverage:0,total_gross_notional:0,total_net_notional:0,positions:1,open_order_count:0,open_orders:[]},execution_assumptions:null}));

  return {totalReturn:(totalReturn*100).toFixed(1),sharpe:sharpe.toFixed(2),maxDD:(maxDD*100).toFixed(1),tradeCount,monthlyCount:monthData.length,equityEnd:eq.toFixed(0)};
}

const a = makeBacktest("sc_a_aggressive", 100000, 0.0012, 0.018, 25);
console.log("Strategy A (aggressive): return="+a.totalReturn+"% sharpe="+a.sharpe+" maxDD="+a.maxDD+"% trades="+a.tradeCount+" equityEnd="+a.equityEnd);

const b = makeBacktest("sc_b_conservative", 100000, 0.0004, 0.010, 12);
console.log("Strategy B (conservative): return="+b.totalReturn+"% sharpe="+b.sharpe+" maxDD="+b.maxDD+"% trades="+b.tradeCount+" equityEnd="+b.equityEnd);
