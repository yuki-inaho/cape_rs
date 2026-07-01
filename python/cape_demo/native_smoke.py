from __future__ import annotations

import argparse
from pathlib import Path

from .extract import extract_frame
from .official_data import load_official_seq_example
from .visualization import save_demo_figure


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the native Rust/PyO3 CAPE demo on official CAPE sample data.")
    parser.add_argument("--out", type=Path, default=None, help="Optional preview PNG output path.")
    args = parser.parse_args()

    scene = load_official_seq_example()
    result = extract_frame(scene)

    plane_count = len(result["planes"])
    cylinder_count = len(result["cylinders"])
    if plane_count < 1 or cylinder_count < 1:
        raise RuntimeError(f"expected at least one plane and cylinder, got {plane_count=} {cylinder_count=}")

    print(f"source={scene.source_dir}")
    print(f"planes={plane_count} cylinders={cylinder_count}")
    print(f"stats={result['stats']}")
    if args.out is not None:
        save_demo_figure(scene, result, args.out)
        print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
