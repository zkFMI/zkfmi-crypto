#!/bin/sh
# Runs inside the hybrid image: same OpenSSL 3.5.5 for every variant.
set -u
O=/opt/pqc-openssl/bin/openssl; W=/tmp/tls; PD=/opt/MP-SPDZ/Player-Data; mkdir -p $W; cd $W
$O version
$O req -x509 -newkey rsa:2048 -nodes -keyout rsa-s.key -out rsa-s.pem -subj /CN=rsa-server -days 2 2>/dev/null
$O req -x509 -newkey rsa:2048 -nodes -keyout rsa-c.key -out rsa-c.pem -subj /CN=rsa-client -days 2 2>/dev/null
$O req -x509 -newkey ed25519 -nodes -keyout ed-s.key -out ed-s.pem -subj /CN=ed-server -days 2 2>/dev/null
$O req -x509 -newkey ed25519 -nodes -keyout ed-c.key -out ed-c.pem -subj /CN=ed-client -days 2 2>/dev/null
cp $PD/P0.pem mldsa-s.pem; cp $PD/P0.key mldsa-s.key; cp $PD/P1.pem mldsa-c.pem; cp $PD/P1.key mldsa-c.key
echo "== certificate DER sizes"
for c in rsa-s ed-s mldsa-s; do printf "%s %s bytes\n" $c "$($O x509 -in $c.pem -outform DER | wc -c)"; done
run() { # name groups
  name=$1; groups=$2
  for mode in server mutual; do
    if [ $mode = mutual ]; then sv="-Verify 1 -CAfile $name-c.pem"; cl="-cert $name-c.pem -key $name-c.key"; else sv=""; cl=""; fi
    $O s_server -accept 14433 -tls1_3 -cert $name-s.pem -key $name-s.key -groups $groups -no_ticket -quiet $sv >/dev/null 2>&1 &
    pid=$!; sleep 1
    echo "== $name $mode groups=$groups"
    $O s_client -connect 127.0.0.1:14433 -brief -CAfile $name-s.pem $cl </dev/null 2>&1 | grep -E "Protocol|group|signature|Peer" | sed 's/^/   /'
    $O s_time -connect 127.0.0.1:14433 -new -time 10 -CAfile $name-s.pem -verify 1 $cl 2>&1 | grep -E "connections in"
    kill $pid; wait $pid 2>/dev/null
  done
}
run rsa X25519
run ed X25519
run mldsa X25519MLKEM768
