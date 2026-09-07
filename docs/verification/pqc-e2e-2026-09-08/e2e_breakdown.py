#!/usr/bin/env python3
"""Six more alternating rounds; after each leg read last_execution.mpc_execution_ms from /api/state."""
import json, statistics, time, urllib.request
V = {"hybrid": 18810, "classical": 18811}
def call(port, path, body=None):
    req = urllib.request.Request(f"http://127.0.0.1:{port}{path}", data=json.dumps(body).encode() if body else None,
                                 headers={"content-type": "application/json"}, method="POST" if body else "GET")
    t0 = time.perf_counter()
    with urllib.request.urlopen(req, timeout=900) as r: d = json.loads(r.read())
    return (time.perf_counter() - t0) * 1000.0, d
rows = []
for rnd in range(6):
    for v, port in V.items():
        for leg, actor, side, tif in (("maker","maker","sell","good_til_cancelled"), ("taker","taker","buy","immediate_or_cancel")):
            wall, _ = call(port, "/api/order", {"actor": actor, "side": side, "price": 100, "quantity": 10, "time_in_force": tif})
            _, st = call(port, "/api/state")
            le = st["data"]["last_execution"] or {}
            mpc = le.get("mpc_execution_ms")
            rows.append((v, leg, wall, mpc)); print(json.dumps({"round": rnd, "variant": v, "leg": leg, "wall_ms": round(wall,1), "mpc_execution_ms": mpc}), flush=True)
print("\nvariant    leg    wall_med  mpc_med  rest_med(wall-mpc)")
for v in V:
    for leg in ("maker","taker"):
        w = [r[2] for r in rows if r[0]==v and r[1]==leg]; m = [r[3] for r in rows if r[0]==v and r[1]==leg]
        rest = [a-b for a,b in zip(w,m)]
        print(f"{v:10} {leg:6} {statistics.median(w):8.1f} {statistics.median(m):8.1f} {statistics.median(rest):8.1f}")
