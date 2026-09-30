"""Package an already-built tagged release; no system libraries are bundled."""
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path


def output(args, cwd=None):
    return subprocess.check_output(args, cwd=cwd, text=True).strip()


def main():
    source, destination = (Path(p).resolve() for p in sys.argv[1:3])
    tag = sys.argv[3]
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        raise ValueError("Expected a version tag such as v0.6.0")
    metadata = json.loads(output([
        "cargo", "metadata", "--locked", "--format-version", "1",
        "--filter-platform", "x86_64-unknown-linux-gnu",
    ], source))
    root_id = metadata["resolve"]["root"]
    packages = {p["id"]: p for p in metadata["packages"]}
    assert packages[root_id]["version"] == tag[1:], "Tag/package version mismatch"
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    pending, reachable = [root_id], set()
    while pending:
        node = pending.pop()
        if node in reachable:
            continue
        reachable.add(node)
        pending.extend(d["pkg"] for d in nodes[node]["deps"])

    destination.mkdir(parents=True, exist_ok=True)
    folder = destination / "Stride-Linux"
    folder.mkdir()  # Refuse to reuse an old package directory.
    executable = folder / "Stride"
    shutil.copy2(source / "target/release/stride", executable)
    executable.chmod(0o755)
    glibc = {
        tuple(map(int, v.split(".")))
        for v in re.findall(r"GLIBC_([\d.]+)", output(["readelf", "--version-info", str(executable)]))
    }
    assert glibc and max(glibc) <= (2, 35), "Binary exceeds glibc 2.35 baseline"
    dependencies = output(["ldd", str(executable)])
    assert "not found" not in dependencies, "Missing build-host runtime dependency"

    templates = Path(__file__).resolve().parent
    shutil.copy2(templates / "Play.sh", folder / "Play.sh")
    (folder / "Play.sh").chmod(0o755)
    (folder / "PLAY.txt").write_text(
        (templates / "PLAY-LINUX.txt").read_text().replace("@VERSION@", tag[1:]),
        encoding="utf-8",
    )
    for name in ("LANDSCAPE.md", "MOVEMENT.md", "COURSE.md", "LICENSE", "THIRD_PARTY.txt", "Cargo.lock"):
        shutil.copy2(source / name, folder / name)
    shutil.copytree(source / "licenses", folder / "licenses")
    notices = ["Additional Linux-target dependency notices (including build dependencies):", ""]
    for package in sorted((packages[i] for i in reachable if i != root_id), key=lambda p: (p["name"], p["version"])):
        label = f'{package["name"]}-{package["version"]}'
        notices.extend([f'{label}: {package.get("license") or "see package notices"}',
                        f'https://crates.io/crates/{package["name"]}/{package["version"]}', ""])
        directory = Path(package["manifest_path"]).parent
        files = set()
        for pattern in ("LICENSE*", "license*", "COPYING*", "NOTICE*", "COPYRIGHT*"):
            for candidate in directory.glob(pattern):
                files.update([candidate] if candidate.is_file() else candidate.rglob("*"))
        if package.get("license_file"):
            files.add(directory / package["license_file"])
        for notice in sorted(p for p in files if p.is_file()):
            name = "-".join(notice.relative_to(directory).parts)
            shutil.copy2(notice, folder / "licenses" / f'{label}-{name}')
    (folder / "LINUX-DEPENDENCIES.txt").write_text("\n".join(notices), encoding="utf-8")
    commit = output(["git", "rev-parse", "HEAD"], source)
    (folder / "SOURCE.txt").write_text(
        f"Stride {tag[1:]} / Linux x86_64\nGame source commit: {commit}\n"
        f"https://github.com/esotericode/Rust3D/tree/{tag}\n"
        "Built on Ubuntu 22.04 with cargo build --release --locked.\n"
        f"{output(['rustc', '--version'])}\n{output(['cargo', '--version'])}\n"
        f"Required glibc symbols: up to {'.'.join(map(str, max(glibc)))}\n"
        "Uses the system's libudev, libasound, X11 and OpenGL; they are not bundled.\n"
        "Cargo.lock and dependency notices are included. Rebuild from the tag.\n",
        encoding="utf-8",
    )
    archive = destination / "Stride-Linux-x86_64.tar.gz"
    with tarfile.open(archive, "w:gz") as tar:
        tar.add(folder, arcname=folder.name)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (destination / "Stride-Linux-x86_64.sha256").write_text(f"{digest}  {archive.name}\n", encoding="ascii")
    print(f"Packaged {archive.name}: {archive.stat().st_size} bytes; source {commit}")
    print(f"SHA256: {digest}")
    print(dependencies)


if __name__ == "__main__":
    main()
