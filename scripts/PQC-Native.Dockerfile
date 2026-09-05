# Deterministic native test image. Build only on softbank-l40s in the dedicated
# integration directory. The parent image is checked by pqc-native-build.sh.
FROM oclob-test:rust-1.97.1-mpspdz-9d809599
USER root
WORKDIR /opt/pqc-build
COPY openssl-3.5.5.tar.gz /opt/pqc-build/openssl-3.5.5.tar.gz
RUN echo 'b28c91532a8b65a1f983b4c28b7488174e4a01008e29ce8e69bd789f28bc2a89  openssl-3.5.5.tar.gz' | sha256sum -c - \
    && tar -xzf openssl-3.5.5.tar.gz \
    && cd openssl-3.5.5 \
    && ./Configure shared no-tests --prefix=/opt/pqc-openssl --openssldir=/etc/ssl \
    && make -j8 build_sw \
    && make install_sw \
    && install -m 0644 LICENSE.txt /opt/pqc-openssl/LICENSE.txt
ENV OPENSSL_DIR=/opt/pqc-openssl
ENV OPENSSL_NO_VENDOR=1
ENV LD_LIBRARY_PATH=/opt/pqc-openssl/lib64:/opt/MP-SPDZ
ENV PATH=/opt/pqc-openssl/bin:${PATH}
ENV PKG_CONFIG_PATH=/opt/pqc-openssl/lib64/pkgconfig
COPY require-hybrid-tls.patch /opt/pqc-build/require-hybrid-tls.patch
RUN test "$(git -C /opt/MP-SPDZ rev-parse HEAD)" = 9d809599ea6ce627216a389ca7d984fbb75d0cb9 \
    && git -C /opt/MP-SPDZ apply --check /opt/pqc-build/require-hybrid-tls.patch \
    && git -C /opt/MP-SPDZ apply /opt/pqc-build/require-hybrid-tls.patch \
    && printf '\nCFLAGS += -I/opt/pqc-openssl/include\nLDLIBS += -L/opt/pqc-openssl/lib64 -Wl,-rpath,/opt/pqc-openssl/lib64\n' >> /opt/MP-SPDZ/CONFIG.mine \
    && make -C /opt/MP-SPDZ clean \
    && make -C /opt/MP-SPDZ -j8 libSPDZ.so malicious-shamir-party.x \
    && openssl version \
    && ldd /opt/MP-SPDZ/libSPDZ.so \
    && ldd /opt/MP-SPDZ/malicious-shamir-party.x
RUN cd /opt/MP-SPDZ \
    && sha256sum Networking/ssl_sockets.h libSPDZ.so malicious-shamir-party.x > .pqc-tls.sha256
# These are isolated single-host regression credentials, never operator keys.
# Real deployments use each owner's local CSR and an independently enrolled CA.
RUN umask 077; cd /opt/MP-SPDZ/Player-Data \
    && for node in 0 1 2 3 4 5 6; do \
        openssl req -x509 -newkey ML-DSA-65 -noenc -days 30 \
          -subj "/CN=P${node}" -addext "subjectAltName=DNS:P${node}" \
          -keyout "P${node}.key" -out "P${node}.pem" || exit 1; \
       done \
    && openssl rehash . \
    && chmod 0600 P[0-6].key
WORKDIR /integration
