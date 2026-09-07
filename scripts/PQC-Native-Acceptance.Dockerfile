# syntax=docker/dockerfile:1.7

# The runner validates this uniquely tagged local image against the fixed
# sha256:91d284... content ID before Docker evaluates this file.
ARG PQC_NATIVE_BASE
FROM ${PQC_NATIVE_BASE} AS integration-builder
USER root
WORKDIR /integration
COPY zkfmi-crypto /integration/zkfmi-crypto
COPY qomm /integration/qomm
COPY zkpi /integration/zkpi
COPY defmi /integration/defmi
COPY oclob /integration/oclob
COPY dekyx /integration/dekyx
COPY deccp /integration/deccp
COPY aethel /integration/aethel
RUN install -d -m 0700 /build/cache/tmp /opt/oclob-bin \
    && cd /opt/MP-SPDZ \
    && sha256sum -c .pqc-tls.sha256
ENV CARGO_TARGET_DIR=/build/target
ENV TMPDIR=/build/cache/tmp
RUN --mount=type=cache,id=pqc-native-acceptance-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=pqc-native-acceptance-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=pqc-native-acceptance-target,target=/build/target \
    cd /integration/oclob \
    && cargo build --locked --release \
       -p oclob-node -p oclob-mpc -p oclob-demo --bins \
    && /build/target/release/oclob-compile-program \
       --root /opt/MP-SPDZ --program oclob_match_v1 \
    && install -m 0755 \
       /build/target/release/oclob-avalanche-acceptance \
       /build/target/release/oclob-node \
       /build/target/release/oclob-lab-provision \
       /build/target/release/oclob-edge-submit \
       /build/target/release/oclob-corporate-worker \
       /build/target/release/oclob-corporate-api \
       /build/target/release/oclob-market-worker \
       /build/target/release/oclob-public-book \
       /build/target/release/oclob-cluster-e2e \
       /build/target/release/oclob-native-bootstrap \
       /build/target/release/oclob-native-e2e \
       /opt/oclob-bin/
RUN --mount=type=cache,id=pqc-native-acceptance-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=pqc-native-acceptance-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=pqc-native-acceptance-target,target=/build/target \
    cd /integration/defmi/rust \
    && env -u MP_SPDZ_ROOT cargo build --locked --release \
       -p defmi-avalanche-vm --bin qomm-avalanche-vm \
    && install -m 0755 /build/target/release/qomm-avalanche-vm /opt/oclob-bin/

FROM ${PQC_NATIVE_BASE} AS avalanche-download
ARG TARGETARCH
ARG AVALANCHEGO_VERSION=1.14.2
ARG ANR_VERSION=1.8.3
USER root
RUN apt-get -o Acquire::ForceIPv4=true -o Acquire::Retries=5 update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /download
RUN set -eu; \
    case "${TARGETARCH}" in \
      amd64) \
        avalanche_sha=daf7f2739f0790c033ad728af6ee29bcd2660baf5265844079eb60f22d101e46; \
        anr_sha=0944cc0d8dd9b893d0f150cbca41a270563ba2ac137403d4cd1658c4e6a00368 ;; \
      arm64) \
        avalanche_sha=0c4563c55659fd9923bd1ea9d3dbcd0a85668325091ffc4a259d0a137033ad0b; \
        anr_sha=60a6306a246c7a89092788fd254c9b574287bb0a83de7b01e196ea5883a4dcfb ;; \
      *) echo "unsupported TARGETARCH=${TARGETARCH}" >&2; exit 2 ;; \
    esac; \
    install -d avalanchego anr; \
    curl --location --fail --silent --show-error \
      "https://github.com/ava-labs/avalanchego/releases/download/v${AVALANCHEGO_VERSION}/avalanchego-linux-${TARGETARCH}-v${AVALANCHEGO_VERSION}.tar.gz" \
      --output avalanchego.tar.gz; \
    echo "${avalanche_sha}  avalanchego.tar.gz" | sha256sum -c -; \
    tar -xzf avalanchego.tar.gz -C avalanchego --strip-components=1; \
    curl --location --fail --silent --show-error \
      "https://github.com/ava-labs/avalanche-network-runner/releases/download/v${ANR_VERSION}/avalanche-network-runner_${ANR_VERSION}_linux_${TARGETARCH}.tar.gz" \
      --output anr.tar.gz; \
    echo "${anr_sha}  anr.tar.gz" | sha256sum -c -; \
    tar -xzf anr.tar.gz -C anr; \
    test -x avalanchego/avalanchego; \
    test -x anr/avalanche-network-runner

FROM ${PQC_NATIVE_BASE} AS oclob-cluster
USER root
RUN apt-get -o Acquire::ForceIPv4=true -o Acquire::Retries=5 update \
    && apt-get install -y --no-install-recommends curl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 oclob \
    && useradd --system --uid 10001 --gid 10001 --home-dir /var/lib/oclob --create-home oclob
COPY --from=integration-builder --chown=oclob:oclob /opt/MP-SPDZ /opt/MP-SPDZ
COPY --from=integration-builder /opt/oclob-bin/oclob-node /usr/local/bin/oclob-node
COPY --from=integration-builder /opt/oclob-bin/oclob-lab-provision /usr/local/bin/oclob-lab-provision
COPY --from=integration-builder /opt/oclob-bin/oclob-edge-submit /usr/local/bin/oclob-edge-submit
COPY --from=integration-builder /opt/oclob-bin/oclob-corporate-worker /usr/local/bin/oclob-corporate-worker
COPY --from=integration-builder /opt/oclob-bin/oclob-corporate-api /usr/local/bin/oclob-corporate-api
COPY --from=integration-builder /opt/oclob-bin/oclob-market-worker /usr/local/bin/oclob-market-worker
COPY --from=integration-builder /opt/oclob-bin/oclob-public-book /usr/local/bin/oclob-public-book
COPY --from=integration-builder /opt/oclob-bin/oclob-cluster-e2e /usr/local/bin/oclob-cluster-e2e
COPY --from=integration-builder /opt/oclob-bin/oclob-native-bootstrap /usr/local/bin/oclob-native-bootstrap
COPY --from=integration-builder /opt/oclob-bin/oclob-native-e2e /usr/local/bin/oclob-native-e2e
RUN ! ldd /opt/MP-SPDZ/malicious-shamir-party.x | grep -F 'not found' \
    && ! ldd /opt/MP-SPDZ/libSPDZ.so | grep -F 'not found' \
    && ! ldd /usr/local/bin/oclob-node | grep -F 'not found' \
    && ! ldd /usr/local/bin/oclob-corporate-api | grep -F 'not found'
ENV MP_SPDZ_ROOT=/opt/MP-SPDZ
ENV LD_LIBRARY_PATH=/opt/pqc-openssl/lib64:/opt/MP-SPDZ
USER oclob
WORKDIR /var/lib/oclob
CMD ["oclob-node", "--config", "/node/config.json"]

FROM ${PQC_NATIVE_BASE} AS oclob-avalanche-acceptance
USER root
RUN apt-get -o Acquire::ForceIPv4=true -o Acquire::Retries=5 update \
    && apt-get install -y --no-install-recommends curl libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system --gid 10001 oclob \
    && useradd --system --uid 10001 --gid 10001 --home-dir /var/lib/oclob --create-home oclob \
    && install -d -o oclob -g oclob -m 0700 /out/tmp
COPY --from=integration-builder --chown=oclob:oclob /opt/MP-SPDZ /opt/MP-SPDZ
COPY --from=integration-builder /opt/oclob-bin/oclob-avalanche-acceptance /usr/local/bin/oclob-avalanche-acceptance
COPY --from=integration-builder /opt/oclob-bin/qomm-avalanche-vm /usr/local/bin/qomm-avalanche-vm
COPY --from=integration-builder /integration/defmi/avalanche/defmivm /opt/defmi/avalanche/defmivm
COPY --from=avalanche-download /download/avalanchego/avalanchego /usr/local/bin/avalanchego
COPY --from=avalanche-download /download/anr/avalanche-network-runner /usr/local/bin/avalanche-network-runner
RUN ! ldd /opt/MP-SPDZ/malicious-shamir-party.x | grep -F 'not found' \
    && ! ldd /opt/MP-SPDZ/libSPDZ.so | grep -F 'not found' \
    && ! ldd /usr/local/bin/oclob-avalanche-acceptance | grep -F 'not found' \
    && ! ldd /usr/local/bin/qomm-avalanche-vm | grep -F 'not found'
ENV AVALANCHEGO_PATH=/usr/local/bin/avalanchego
ENV AVALANCHE_NETWORK_RUNNER=/usr/local/bin/avalanche-network-runner
ENV QOMM_AVALANCHE_ACCEPTANCE_BIN=/usr/local/bin/oclob-avalanche-acceptance
ENV QOMM_AVALANCHE_VM_BIN=/usr/local/bin/qomm-avalanche-vm
ENV QOMM_AVALANCHE_ARTIFACT=/out/oclob_avalanche_acceptance.json
ENV MP_SPDZ_ROOT=/opt/MP-SPDZ
ENV LD_LIBRARY_PATH=/opt/pqc-openssl/lib64:/opt/MP-SPDZ
ENV TMPDIR=/out/tmp
USER oclob
WORKDIR /var/lib/oclob
ENTRYPOINT ["/opt/defmi/avalanche/defmivm/scripts/run-local-l1.sh"]
