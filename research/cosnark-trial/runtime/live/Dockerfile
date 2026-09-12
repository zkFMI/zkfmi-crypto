ARG BASE_IMAGE=oclob-pqc-native-avalanche:20260908t071221z-32876-pqc-native-acceptance
FROM ${BASE_IMAGE}

# AvalancheGo intentionally starts plugins without the parent's loader
# environment. Register the exact libraries already shipped in the pinned
# image; do not change the proof verifier or add a plugin wrapper.
USER 0
COPY 00-qomm-runtime.conf /etc/ld.so.conf.d/00-qomm-runtime.conf
RUN ldconfig
USER 10001:10001
