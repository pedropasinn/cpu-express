#!/usr/bin/env python3
"""Collect original notices from a resolved Cargo graph and optional toolchain.

No Cargo command or compilation is run. Pass an explicit metadata JSON produced
with --filter-platform TARGET, matching --target. Files are copied byte-for-byte;
SPDX expressions come from Cargo metadata and versions are checked against lock.
"""

from __future__ import annotations

import argparse
import concurrent.futures
import hashlib
import json
import re
import shutil
import tempfile
import tomllib
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path


GENERATOR = "cpu-express-collect-licenses-v1"
LICENSE_NAME = re.compile(r"^(?:licen[cs]e|copying|copyright|notice|ofl|ufl|unlicense)", re.I)
CODE_LICENSE_NAME = re.compile(r"^(?:licen[cs]e|copying|unlicense)", re.I)
EXCLUDED_SUFFIXES = {".rs", ".c", ".h", ".js", ".py", ".yml", ".yaml", ".toml", ".json"}
UPSTREAM_FILES = ("LICENSE-MIT", "LICENSE-APACHE", "LICENSE", "NOTICE")


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def markdown(text: str) -> str:
    return text.replace("|", "\\|").replace("\n", " ")


def local_notices(root: Path, license_file: str | None) -> list[Path]:
    files = set()
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix.lower() in EXCLUDED_SUFFIXES:
            continue
        relative = path.relative_to(root)
        in_licenses = any(part.lower() in {"license", "licenses", "licences"} for part in relative.parts[:-1])
        font_notice = "fonts" in relative.parts and path.suffix.lower() == ".txt"
        if LICENSE_NAME.match(path.name) or in_licenses or font_notice:
            files.add(path)
    if license_file:
        path = (root / license_file).resolve()
        if not path.is_relative_to(root.resolve()):
            raise ValueError(f"license_file escapes package: {root.name}")
        if not path.is_file():
            raise FileNotFoundError(f"Declared license_file missing: {root.name}/{license_file}")
        files.add(path)
    return sorted(files)


def fetch(url: str, cache: Path | None) -> bytes | None:
    key = sha256(url.encode())
    cached = cache / key if cache else None
    if cached and cached.is_file():
        return cached.read_bytes()
    if cached and cached.with_suffix(".404").is_file():
        return None
    request = urllib.request.Request(url, headers={"User-Agent": "CPU-Express-license-inventory/0.1"})
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            data = response.read()
    except urllib.error.HTTPError as error:
        if error.code != 404:
            raise
        if cached:
            cached.with_suffix(".404").write_text(url + "\n")
        return None
    if cached:
        cached.write_bytes(data)
    return data


def github_repository(repository: str | None) -> str | None:
    if not repository:
        return None
    parsed = urllib.parse.urlparse(repository)
    parts = parsed.path.strip("/").split("/")
    if parsed.hostname != "github.com" or len(parts) < 2:
        return None
    return f"https://github.com/{parts[0]}/{parts[1].removesuffix('.git')}"


def collect(args: argparse.Namespace) -> dict:
    metadata_path = args.metadata.resolve()
    metadata_bytes = metadata_path.read_bytes()
    metadata = json.loads(metadata_bytes)
    lock_bytes = args.lockfile.read_bytes()
    lock = tomllib.loads(lock_bytes.decode())
    locked = {(p["name"], p["version"], p.get("source")) for p in lock["package"]}
    ids = {node["id"] for node in metadata["resolve"]["nodes"]}
    packages = sorted(
        (p for p in metadata["packages"] if p["id"] in ids and (p.get("source") or "").startswith("registry+")),
        key=lambda p: (p["name"], p["version"]),
    )
    overrides = json.loads(args.source_overrides.read_text()) if args.source_overrides else {"overrides": []}
    override_map = {(x["repository"], x["published_revision"]): x for x in overrides["overrides"]}
    args.output.mkdir(parents=True, exist_ok=True)
    if args.cache_dir:
        args.cache_dir.mkdir(parents=True, exist_ok=True)
    existing = args.output / "licenses"
    if existing.exists():
        manifest_path = existing / "manifest.json"
        if not manifest_path.is_file() or json.loads(manifest_path.read_text()).get("generator") != GENERATOR:
            raise ValueError("Refusing to replace a licenses directory not created by this script")

    result = {
        "generator": GENERATOR,
        "target": args.target,
        "metadata_sha256": sha256(metadata_bytes),
        "cargo_lock_sha256": sha256(lock_bytes),
        "packages": [],
        "toolchain_files": [],
    }
    upstream_cache = {}
    with tempfile.TemporaryDirectory(prefix=".licenses-", dir=args.output) as temp:
        staging = Path(temp) / "licenses"
        staging.mkdir()

        def copy_bytes(data: bytes, relative: Path, origin: str) -> dict:
            destination = staging / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(data)
            return {"path": relative.as_posix(), "source": origin, "bytes": len(data), "sha256": sha256(data)}

        for package in packages:
            name, version = package["name"], package["version"]
            if (name, version, package.get("source")) not in locked:
                raise ValueError(f"Metadata package not in Cargo.lock: {name} {version}")
            root = Path(package["manifest_path"]).parent
            base = Path("crates") / f"{name}-{version}"
            record = {"name": name, "version": version, "license": package.get("license"),
                      "repository": package.get("repository"), "registry": package["source"],
                      "authors": package.get("authors", []), "files": []}
            selected = local_notices(root, package.get("license_file"))
            for path in selected:
                relative = path.relative_to(root)
                record["files"].append(copy_bytes(path.read_bytes(), base / relative,
                    f"Cargo registry package {name}-{version}/{relative.as_posix()}"))
            # Font licenses alone do not supply the Rust crate's MIT/Apache texts.
            has_code_license = any(CODE_LICENSE_NAME.match(p.name) and "fonts" not in p.relative_to(root).parts
                                   for p in selected)
            if not has_code_license:
                if not args.fetch_missing:
                    raise ValueError(f"No crate-level license text for {name}; rerun with --fetch-missing")
                vcs_path = root / ".cargo_vcs_info.json"
                vcs = json.loads(vcs_path.read_text()) if vcs_path.is_file() else {}
                published = vcs.get("git", {}).get("sha1")
                repository = github_repository(package.get("repository"))
                if not repository:
                    raise ValueError(f"Cannot identify pinned upstream license source for {name}")
                override = override_map.get((repository, published))
                if override and override.get("package_name") not in (None, name):
                    override = None
                if override and override.get("package_version") not in (None, version):
                    override = None
                if not published and not override:
                    raise ValueError(f"Cannot identify pinned upstream license source for {name}")
                revision = override["replacement_revision"] if override else published
                record["upstream_license_revision"] = revision
                record["published_vcs_revision"] = published
                if override:
                    record["upstream_source_note"] = override
                key = (repository, revision)
                if key not in upstream_cache:
                    raw = repository.replace("https://github.com/", "https://raw.githubusercontent.com/")
                    urls = [(file, f"{raw}/{revision}/{file}") for file in UPSTREAM_FILES]
                    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
                        fetched = list(pool.map(lambda item: fetch(item[1], args.cache_dir), urls))
                    upstream_cache[key] = [(file, url, data) for (file, url), data in zip(urls, fetched) if data]
                remote_files = upstream_cache[key]
                if not any(CODE_LICENSE_NAME.match(file) for file, _, _ in remote_files):
                    raise ValueError(f"No upstream license at {repository}@{revision} for {name}")
                for file, url, data in remote_files:
                    record["files"].append(copy_bytes(data, base / "upstream" / file, url))
            result["packages"].append(record)

        if args.rust_doc_dir:
            rust_root = args.rust_doc_dir
            paths = [rust_root / "COPYRIGHT-library.html", *sorted((rust_root / "licenses").glob("*"))]
            for path in paths:
                if not path.is_file():
                    raise FileNotFoundError(path)
                relative = path.relative_to(rust_root)
                result["toolchain_files"].append(copy_bytes(path.read_bytes(), Path("toolchain/rust") / relative,
                    f"Rust {args.rust_version} distribution share/doc/rust/{relative.as_posix()}"))

        if args.mingw_doc_dir:
            for package in ("mingw-w64-x86-64-dev", "mingw-w64-common", "gcc-mingw-w64-x86-64-posix-runtime",
                            "gcc-mingw-w64-base", "binutils-mingw-w64-x86-64"):
                path = args.mingw_doc_dir / package / "copyright"
                if not path.is_file():
                    raise FileNotFoundError(path)
                result["toolchain_files"].append(copy_bytes(path.read_bytes(), Path("toolchain/mingw") / package / "copyright.txt",
                    f"Debian package {package}, usr/share/doc/{package}/copyright"))
            if args.common_licenses_dir:
                for path in sorted(args.common_licenses_dir.iterdir()):
                    if path.is_file():
                        result["toolchain_files"].append(copy_bytes(path.read_bytes(), Path("toolchain/mingw/common-licenses") / path.name,
                            f"Debian common license usr/share/common-licenses/{path.name}"))

        result["package_count"] = len(result["packages"])
        result["notice_count"] = sum(len(p["files"]) for p in result["packages"]) + len(result["toolchain_files"])
        (staging / "manifest.json").write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n")
        if existing.exists():
            shutil.rmtree(existing)
        shutil.move(str(staging), str(existing))

    lines = ["# Avisos e licenças de terceiros — CPU Express", "",
        f"Inventário de **{len(packages)} pacotes de registry** do grafo Cargo resolvido para `{args.target}`.", "",
        "Versões conferidas contra Cargo.lock. As expressões SPDX abaixo reproduzem o campo `license` do metadata; "
        "os textos de licença e avisos foram preservados sem alteração. Este grafo inclui dependências de compilação "
        "e pode incluir pacotes que não contribuam código ao executável; a lista não é uma análise de símbolos do binário.", "",
        "O código próprio do CPU Express e seus desenhos procedurais estão cobertos por [LICENSE](LICENSE). "
        "Nenhum código, sprite ou áudio do APK de Tiny Rails foi incorporado ao jogo.", "",
        "## Pacotes Cargo", "", "| Pacote | Versão | Expressão de licença declarada | Textos e avisos |",
        "|---|---|---|---|"]
    for package in result["packages"]:
        crate_url = f"https://crates.io/crates/{package['name']}/{package['version']}"
        links = [f"[{markdown(Path(file['path']).name)}](licenses/{file['path']})" for file in package["files"]]
        lines.append(f"| [{package['name']}]({crate_url}) | {package['version']} | {markdown(package['license'] or 'license_file')} | {' · '.join(links)} |")
    lines.extend(["", "## Fontes embutidas", "",
        "Os avisos originais da crate `epaint_default_fonts` incluem os textos OFL (Noto), UFL (Ubuntu), "
        "Hack-Regular, egui-icons e emoji-icon-font. Estão na subpasta `fonts` correspondente em `licenses/crates/`; "
        "foram preservados junto aos textos da licença do código Rust.", "", "## Origem dos textos complementares", "",
        "Quando o pacote publicado não continha sua licença de código, o coletor usou o repositório upstream no "
        "commit declarado por `.cargo_vcs_info.json`. URLs, revisões, nomes, tamanhos e SHA-256 de cada texto estão "
        "em [licenses/manifest.json](licenses/manifest.json). Arquivos NOTICE/COPYRIGHT encontrados nos pacotes "
        "e NOTICE encontrado na consulta upstream também são incluídos.", ""])
    notes = {json.dumps(p["upstream_source_note"], sort_keys=True) for p in result["packages"] if "upstream_source_note" in p}
    for item in sorted(notes):
        note = json.loads(item)
        lines.append(f"- **Origem alternativa documentada:** {note['reason']} Fonte: [{note['repository']}]({note['repository']}/tree/{note['replacement_revision']}).")
    lines.extend(["", "## Biblioteca padrão Rust e toolchain Windows", "",
        "Os avisos distribuídos com a biblioteca padrão Rust estão em [COPYRIGHT-library.html](licenses/toolchain/rust/COPYRIGHT-library.html), "
        "com os textos da pasta `licenses` da distribuição. Os avisos Debian de MinGW, GCC runtime e binutils "
        "estão em `licenses/toolchain/mingw`, incluindo os textos comuns referenciados por esses avisos. "
        "Os arquivos abrangem os pacotes de ferramentas; sua inclusão não afirma que toda ferramenta foi incorporada ao CPU Express.", "",
        "O copyright do pacote GCC inclui o texto da GCC Runtime Library Exception. Os documentos originais foram "
        "mantidos completos, sem substituir as condições por um resumo.", "", "## Reproduzir o inventário", "", "```bash",
        "cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-gnu > cargo-metadata-windows.json",
        "python3 scripts/collect-licenses.py --metadata cargo-metadata-windows.json --fetch-missing \\",
        "  --source-overrides scripts/license-source-overrides.json \\",
        "  --rust-doc-dir <toolchain>/share/doc/rust --rust-version 1.100.0 \\",
        "  --mingw-doc-dir <mingw>/usr/share/doc --common-licenses-dir /usr/share/common-licenses",
        "```", "", "Python 3.11 ou posterior. O script não compila nem instala pacotes; lê o metadata fornecido, "
        "confere Cargo.lock e copia os avisos. `--fetch-missing` permite buscar somente os textos faltantes em "
        "fontes upstream fixadas por revisão. Os caminhos de toolchain são argumentos, sem dependência de um "
        "diretório pessoal específico.", ""])
    (args.output / "THIRD_PARTY_NOTICES.md").write_text("\n".join(lines))
    return result


def main() -> None:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--lockfile", type=Path, default=root / "Cargo.lock")
    parser.add_argument("--output", type=Path, default=root)
    parser.add_argument("--target", default="x86_64-pc-windows-gnu")
    parser.add_argument("--fetch-missing", action="store_true")
    parser.add_argument("--cache-dir", type=Path)
    parser.add_argument("--source-overrides", type=Path)
    parser.add_argument("--rust-doc-dir", type=Path)
    parser.add_argument("--rust-version", default="unspecified")
    parser.add_argument("--mingw-doc-dir", type=Path)
    parser.add_argument("--common-licenses-dir", type=Path)
    args = parser.parse_args()
    result = collect(args)
    print(json.dumps({"packages": result["package_count"], "notice_files": result["notice_count"],
                      "cargo_lock_sha256": result["cargo_lock_sha256"]}, indent=2))


if __name__ == "__main__":
    main()
