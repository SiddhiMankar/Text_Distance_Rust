#!/usr/bin/env python3
"""
Differential Fuzz Harness Driver for textdistance (Python) vs textdistancerust (Rust).
Communicates with textdistancerust-cli over persistent stdin/stdout JSON IPC streaming.
"""

import argparse
import json
import math
import os
import subprocess
import sys
import time
from pathlib import Path

# Add textdistance parent directory to sys.path
sys.path.insert(0, str(Path(__file__).parent.parent / 'textdistance'))
import textdistance


class RustProcess:
    def __init__(self, bin_path: str):
        self.proc = subprocess.Popen(
            [bin_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            encoding='utf-8',
            bufsize=1,
        )

    def query(self, alg: str, s1: str, s2: str, mat=None) -> dict:
        payload = {"alg": alg, "s1": s1, "s2": s2}
        if mat is not None:
            payload["mat"] = mat
        req = json.dumps(payload)
        self.proc.stdin.write(req + '\n')
        self.proc.stdin.flush()
        line = self.proc.stdout.readline()
        if not line:
            raise RuntimeError("Rust binary stdout closed unexpectedly")
        return json.loads(line)

    def close(self):
        self.proc.stdin.close()
        self.proc.terminate()
        self.proc.wait()


def run_seed_tests(rust_proc: RustProcess, alg_name: str):
    print(f"Running seed corpus tests for '{alg_name}'...")
    seed_cases = [
        ("", ""),
        ("", "a"),
        ("a", ""),
        ("a", "a"),
        ("a", "b"),
        ("spam", "qwer"),
        ("hello", "world"),
        ("test", "test"),
        ("short", "longer text here"),
    ]
    for s1, s2 in seed_cases:
        verify_pair(rust_proc, alg_name, s1, s2)

    if alg_name == "matrix":
        # Additional custom matrix seed cases
        mat_cases = [
            ("cat", "bat", [["cat", "bat", 0.5]]),
            ("bat", "cat", [["cat", "bat", 0.5]]),
            ("dog", "fox", [["cat", "bat", 0.5]]),
            ("dog", "dog", [["cat", "bat", 0.5]]),
        ]
        for s1, s2, mat in mat_cases:
            verify_pair(rust_proc, alg_name, s1, s2, mat=mat)

    print(f"Seed corpus tests passed for '{alg_name}'.")


def verify_pair(rust_proc: RustProcess, alg_name: str, s1: str, s2: str, mat=None):
    if mat is not None:
        py_mat_dict = {(item[0], item[1]): item[2] for item in mat}
        py_alg = textdistance.Matrix(mat=py_mat_dict)
    else:
        py_alg = getattr(textdistance, alg_name)

    if alg_name in ("prefix", "postfix"):
        # Track 1: Verify literal substring output (__call__)
        py_subseq = py_alg(s1, s2)
        rust_res = rust_proc.query(alg_name, s1, s2, mat=mat)
        if rust_res.get("error"):
            raise ValueError(f"Rust error for {alg_name}({repr(s1)}, {repr(s2)}): {rust_res['error']}")
        
        r_subseq = rust_res.get("subsequence")
        if r_subseq is not None and r_subseq != py_subseq:
            raise AssertionError(f"Mismatch in {alg_name}.subsequence({repr(s1)}, {repr(s2)}): Py={repr(py_subseq)}, Rust={repr(r_subseq)}")

        # Track 2: Verify derived numeric fields against Py similarity and distance
        py_sim = py_alg.similarity(s1, s2)
        py_dist = py_alg.distance(s1, s2)
        py_norm_sim = py_alg.normalized_similarity(s1, s2)
        py_norm_dist = py_alg.normalized_distance(s1, s2)

        check_close("similarity", py_sim, rust_res["similarity"], alg_name, s1, s2)
        check_close("distance", py_dist, rust_res["distance"], alg_name, s1, s2)
        check_close("normalized_similarity", py_norm_sim, rust_res["normalized_similarity"], alg_name, s1, s2)
        check_close("normalized_distance", py_norm_dist, rust_res["normalized_distance"], alg_name, s1, s2)
        return

    py_sim = py_alg.similarity(s1, s2)
    py_dist = py_alg.distance(s1, s2)
    py_norm_sim = py_alg.normalized_similarity(s1, s2)
    py_norm_dist = py_alg.normalized_distance(s1, s2)

    rust_res = rust_proc.query(alg_name, s1, s2, mat=mat)
    if rust_res.get("error"):
        raise ValueError(f"Rust error for {alg_name}({repr(s1)}, {repr(s2)}): {rust_res['error']}")

    r_sim = rust_res["similarity"]
    r_dist = rust_res["distance"]
    r_norm_sim = rust_res["normalized_similarity"]
    r_norm_dist = rust_res["normalized_distance"]

    check_close("similarity", py_sim, r_sim, alg_name, s1, s2)
    check_close("distance", py_dist, r_dist, alg_name, s1, s2)
    check_close("normalized_similarity", py_norm_sim, r_norm_sim, alg_name, s1, s2)
    check_close("normalized_distance", py_norm_dist, r_norm_dist, alg_name, s1, s2)


def check_close(name: str, py_val, rust_val, alg: str, s1: str, s2: str):
    if math.isinf(py_val):
        if math.isinf(rust_val) and (py_val > 0) == (rust_val > 0):
            return
        raise AssertionError(f"Mismatch in {alg}.{name}({repr(s1)}, {repr(s2)}): Py={py_val}, Rust={rust_val}")
    if math.isnan(py_val):
        if math.isnan(rust_val):
            return
        raise AssertionError(f"Mismatch in {alg}.{name}({repr(s1)}, {repr(s2)}): Py={py_val}, Rust={rust_val}")
    if not math.isclose(py_val, rust_val, abs_tol=1e-9):
        raise AssertionError(f"Mismatch in {alg}.{name}({repr(s1)}, {repr(s2)}): Py={py_val}, Rust={rust_val}")


def main():
    default_bin = Path(__file__).parent.parent / "textdistancerust" / "target" / "release" / "textdistancerust-cli.exe"
    parser = argparse.ArgumentParser()
    parser.add_argument("--alg", required=True, help="Comma-separated algorithm names")
    parser.add_argument("--iterations", type=int, default=10000)
    parser.add_argument("--bin", default=str(default_bin))
    args = parser.parse_args()

    bin_path = str(Path(args.bin).resolve())
    if not os.path.exists(bin_path):
        debug_bin = Path(bin_path.replace("release", "debug"))
        if debug_bin.exists():
            bin_path = str(debug_bin)

    print(f"Connecting to persistent Rust process: {bin_path}")
    rust_proc = RustProcess(bin_path)

    import hypothesis
    import hypothesis.strategies as st

    algs = [a.strip() for a in args.alg.split(",")]
    for alg_name in algs:
        run_seed_tests(rust_proc, alg_name)

        print(f"Fuzzing '{alg_name}' with {args.iterations} iterations...")
        start_time = time.time()

        if alg_name == "matrix":
            @hypothesis.settings(max_examples=args.iterations, deadline=None)
            @hypothesis.given(
                s1=st.text(),
                s2=st.text(),
                mat_entries=st.lists(
                    st.tuples(
                        st.text(),
                        st.text(),
                        st.floats(min_value=-100.0, max_value=100.0, allow_nan=False, allow_infinity=False),
                    ),
                    max_size=5,
                ),
            )
            def fuzz_matrix(s1, s2, mat_entries):
                mat_param = [list(e) for e in mat_entries] if mat_entries else None
                verify_pair(rust_proc, alg_name, s1, s2, mat=mat_param)

            fuzz_matrix()
        else:
            @hypothesis.settings(max_examples=args.iterations, deadline=None)
            @hypothesis.given(s1=st.text(), s2=st.text())
            def fuzz_test(s1, s2):
                verify_pair(rust_proc, alg_name, s1, s2)

            fuzz_test()

        elapsed = time.time() - start_time
        print(f"PASSED {args.iterations} iterations for '{alg_name}' in {elapsed:.2f}s (0 mismatches).")

    rust_proc.close()


if __name__ == "__main__":
    main()
