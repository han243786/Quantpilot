import json, urllib.request, sys, os

qs_path = sys.argv[1] if len(sys.argv) > 1 else "tests/scenarios/scenario_01_btc_dual_ma.qs"
with open(qs_path, 'r', encoding='utf-8') as f:
    source = f.read()

data = json.dumps({'source': source}).encode('utf-8')
req = urllib.request.Request(
    'http://127.0.0.1:3000/api/test/scenario/run',
    data=data,
    headers={'Content-Type': 'application/json'}
)
try:
    resp = urllib.request.urlopen(req, timeout=120)
    result = json.loads(resp.read().decode('utf-8'))
    print(json.dumps(result, indent=2, ensure_ascii=False))
except Exception as e:
    print(f'Error: {e}')
    if hasattr(e, 'read'):
        body = e.read().decode('utf-8')
        print(f'Response: {body}')
