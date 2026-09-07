#!/bin/bash
set -u
D=~/work/pqc-e2e-20260908; cd $D
echo "start $(date -u +%FT%TZ) host=$(hostname) load=$(cut -d' ' -f1-3 /proc/loadavg)"
docker rm -f oclob-e2e-hybrid oclob-e2e-classical oclob-e2e-tls >/dev/null 2>&1
PASS=$(openssl rand -hex 24)
docker run -d --name oclob-e2e-hybrid -p 127.0.0.1:18810:18800 -e OCLOB_QUEUE_PASSPHRASE=$PASS -v oclob-e2e-hybrid-20260908:/var/lib/oclob oclob-server:main-20260907
docker run -d --name oclob-e2e-classical -p 127.0.0.1:18811:18800 -e OCLOB_QUEUE_PASSPHRASE=$PASS -v oclob-e2e-classical-20260908:/var/lib/oclob oclob-server:20260907
for p in 18810 18811; do
  t0=$(date +%s); until curl -sf -m 3 127.0.0.1:$p/health >/dev/null; do sleep 2; [ $(( $(date +%s) - t0 )) -gt 900 ] && { echo "port $p never became healthy"; docker logs --tail 30 oclob-e2e-hybrid oclob-e2e-classical; exit 1; }; done
  echo "port $p healthy after $(( $(date +%s) - t0 )) s"
done
docker image inspect oclob-server:main-20260907 oclob-server:20260907 --format '{{.RepoTags}} {{.Id}}'
echo "== e2e rounds $(date -u +%FT%TZ)"
python3 e2e_bench.py e2e-records.jsonl
echo "== tls handshakes $(date -u +%FT%TZ)"
docker run -d --name oclob-e2e-tls --entrypoint sh oclob-server:main-20260907 -c "sleep 3600" >/dev/null
docker cp tls_bench.sh oclob-e2e-tls:/tls_bench.sh
docker exec oclob-e2e-tls sh /tls_bench.sh
docker rm -f oclob-e2e-tls >/dev/null
echo "end $(date -u +%FT%TZ) load=$(cut -d' ' -f1-3 /proc/loadavg)"
