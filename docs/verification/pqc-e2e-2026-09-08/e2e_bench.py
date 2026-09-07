#!/usr/bin/env python3
"""Alternate order rounds between the two deployed OCLOB images and record wall time."""
import json, statistics, sys, time, urllib.request

VARIANTS = {"hybrid": 18810, "classical": 18811}
WARMUP, ROUNDS = 2, 20
OUT = sys.argv[1]

def post(port, body):
    req = urllib.request.Request(f"http://127.0.0.1:{port}/api/order", data=json.dumps(body).encode(),
                                 headers={"content-type": "application/json"}, method="POST")
    t0 = time.perf_counter()
    with urllib.request.urlopen(req, timeout=900) as r:
        data = json.loads(r.read())
    return (time.perf_counter() - t0) * 1000.0, data

def find(obj, key):
    if isinstance(obj, dict):
        if key in obj:
            return obj[key]
        for v in obj.values():
            hit = find(v, key)
            if hit is not None:
                return hit
    elif isinstance(obj, list):
        for v in obj:
            hit = find(v, key)
            if hit is not None:
                return hit
    return None

records = []
with open(OUT, "w") as out:
    for rnd in range(WARMUP + ROUNDS):
        for variant, port in VARIANTS.items():
            for leg, actor, side, tif in (("maker", "maker", "sell", "good_til_cancelled"),
                                          ("taker", "taker", "buy", "immediate_or_cancel")):
                wall, data = post(port, {"actor": actor, "side": side, "price": 100, "quantity": 10, "time_in_force": tif})
                worker = data.get("data", {}).get("worker", {})
                fills = find(worker, "fills")
                rec = {"round": rnd, "warmup": rnd < WARMUP, "variant": variant, "leg": leg, "wall_ms": round(wall, 1),
                       "execution_ms": find(worker, "execution_ms"), "fills": len(fills) if isinstance(fills, list) else None,
                       "status": worker.get("status"), "ok": data.get("ok")}
                records.append(rec)
                out.write(json.dumps(rec) + "\n"); out.flush()
                print(json.dumps(rec), flush=True)

def p95(xs):
    xs = sorted(xs); return xs[max(0, int(round(0.95 * len(xs))) - 1)]

print("\n== summary (measured rounds only) ==")
print(f"{'variant':10}{'leg':7}{'n':>3}{'wall med':>10}{'wall p95':>10}{'wall mean':>11}{'exec med':>10}{'exec p95':>10}{'fills':>6}")
summary = {}
for variant in VARIANTS:
    for leg in ("maker", "taker"):
        rs = [r for r in records if not r["warmup"] and r["variant"] == variant and r["leg"] == leg]
        w = [r["wall_ms"] for r in rs]; e = [r["execution_ms"] for r in rs if r["execution_ms"] is not None]
        fills = sum(r["fills"] or 0 for r in rs)
        summary[(variant, leg)] = {"n": len(rs), "wall_median": statistics.median(w), "wall_p95": p95(w), "wall_mean": statistics.mean(w),
                                   "exec_median": statistics.median(e) if e else None, "exec_p95": p95(e) if e else None, "fills": fills}
        s = summary[(variant, leg)]
        print(f"{variant:10}{leg:7}{s['n']:>3}{s['wall_median']:>10.1f}{s['wall_p95']:>10.1f}{s['wall_mean']:>11.1f}"
              f"{(s['exec_median'] if s['exec_median'] is not None else float('nan')):>10.1f}"
              f"{(s['exec_p95'] if s['exec_p95'] is not None else float('nan')):>10.1f}{fills:>6}")
for leg in ("maker", "taker"):
    h, c = summary[("hybrid", leg)], summary[("classical", leg)]
    print(f"{leg}: hybrid - classical, wall median {h['wall_median'] - c['wall_median']:+.1f} ms, p95 {h['wall_p95'] - c['wall_p95']:+.1f} ms")
