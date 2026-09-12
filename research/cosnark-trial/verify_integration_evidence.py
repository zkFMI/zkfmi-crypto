#!/usr/bin/env python3
"""Read back public artifacts and executed source identities; never read shares.

Run on the isolated remote host after integration-full completes. Private
engine access is restricted here to public compiler/program/build artifacts.
This deterministic evidence check does not launch or promote an experiment.
"""
import argparse
import hashlib
import json
from pathlib import Path


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def read(path):
    return json.loads(path.read_text())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("legacy_root", type=Path)
    parser.add_argument("run")
    parser.add_argument("dependency_reference", type=Path)
    args = parser.parse_args()
    root, legacy = args.root, args.legacy_root
    trial = root / "src/zkfmi-crypto/research/cosnark-trial"
    public = root / "public" / args.run
    receipt = read(public / "receipt.json")
    assert receipt["verdict"] == "smoke_only" and receipt["primary_metric_value"] == 1
    assert receipt["error"] is None
    assert all(receipt["proof_negative_gates"].values())
    canonical = read(public / "canonical-receipt.json")
    assert canonical == receipt["canonical_receipt"] and canonical["accepted"]
    assert all(canonical["negative_gates"].values())
    identities = {}

    def check(path, expected=None):
        digest = sha(path)
        if expected is not None:
            assert digest == expected, str(path)
        identities[str(path)] = digest

    for path, digest in receipt["source_sha256"].items():
        check(trial / path, digest)
    check(trial / f"integration-experiment-{args.run}.json", receipt["manifest_sha256"])
    check(legacy / "target/release/zkfmi-cosnark-trial", receipt["binary_sha256"])
    check(root / "defmi-target/release/cocode-canonical-acceptance", receipt["canonical_binary_sha256"])
    for path in sorted((root / "src/defmi/rust").rglob("*.rs")):
        if "target" not in path.parts:
            check(path)
    for path in sorted((root / "src/defmi/rust").rglob("Cargo.*")):
        if "target" not in path.parts:
            check(path)
    for proof in receipt["proofs"]:
        operation = proof["operation"]
        check(public / operation / "proof.json", proof["proof_sha256"])
        assert (public / operation / "proof.json").stat().st_size == proof["proof_bytes"]
        assert read(public / operation / "statement.json") == proof["statement"]
        assert proof["owner_query_replay_rejected"] == [True] * 7
    recovery = canonical["pending_upload_recovery"]
    assert recovery["state_readback_matches"] and recovery["state_sync_readback_matches"]
    assert recovery["proof_uploaded_chunks"] == recovery["proof_expected_chunks"] > 0
    check(public / "canonical/fill-pending-state.json", recovery["state_sha256"])
    check(public / "canonical/fill-pending-snapshot.bin", recovery["state_sync_snapshot_sha256"])
    check(public / "canonical/fill-pending-summary.bin", recovery["state_sync_summary_sha256"])
    native_counts = {}
    for operation, filename in (("fill", "fill/native.json"), ("nofill", "nofill/native.json"), ("invalid", "invalid-native.json")):
        records = read(public / filename)
        engine = root / "private" / args.run / operation
        for record in records:
            check(engine / "Programs/Source" / (record["program"] + ".mpc"), record["source_sha256"])
            for path, digest in record["compiled_artifacts"]:
                check(engine / path, digest)
                if path.endswith(".sch"):
                    assert "sec:128" in (engine / path).read_text().replace(" ", "")
            expected = [1] * 7 if operation == "invalid" else [0] * 7
            assert record["party_exit_codes"] == expected
            if operation == "invalid":
                assert record["rejection_reason"] == "invalid financial witness"
            else:
                assert record["seven_context_views_equal"] and record["rejection_reason"] is None
        native_counts[operation] = {"rounds": len(records), "party_exits": 7 * len(records)}
    dependencies = read(args.dependency_reference)
    for path, digest in dependencies["sha256"].items():
        if path.startswith("engine-012/"):
            current = root / "private" / args.run / "fill" / path.removeprefix("engine-012/")
        elif path.startswith("src/"):
            current = root / path
        else:
            current = legacy / path
        check(current, digest)
    check(public / "receipt.json")
    check(public / "canonical-receipt.json")
    print(json.dumps({"verified": True, "experiment_id": receipt["experiment_id"],
        "contract_id": receipt["contract_id"], "contract_sha256": receipt["contract_sha256"],
        "stage": receipt["stage"], "verdict": receipt["verdict"],
        "native_counts": native_counts, "dependency_reference_matches": len(dependencies["sha256"]),
        "pending_upload_recovery": recovery, "sha256": identities}, indent=2))


if __name__ == "__main__":
    main()
