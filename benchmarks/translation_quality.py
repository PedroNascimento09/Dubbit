#!/usr/bin/env python3
import argparse
import json
import pathlib
import random
import time
import urllib.request
import zipfile

from argos_engine import translator
from sacrebleu.metrics import BLEU, CHRF

ROOT = pathlib.Path(__file__).resolve().parent.parent
CACHE = ROOT / ".tools" / "benchmarks"
CORPORA = {
    "tatoeba": "https://object.pouta.csc.fi/OPUS-Tatoeba/v2023-04-12/moses/en-pt.txt.zip",
    "wikimedia": "https://object.pouta.csc.fi/OPUS-wikimedia/v20230407/moses/en-pt.txt.zip",
}
MINIMUM = 20
MAXIMUM = 200


def prepare(name: str) -> pathlib.Path:
    folder = CACHE / f"{name}-en-pt"
    if folder.is_dir() and any(folder.glob("*.en")):
        return folder
    archive = CACHE / f"{name}-en-pt.zip"
    CACHE.mkdir(parents=True, exist_ok=True)
    if not archive.is_file():
        print(f"downloading {CORPORA[name]}")
        urllib.request.urlretrieve(CORPORA[name], archive)
    with zipfile.ZipFile(archive) as bundle:
        bundle.extractall(folder)
    return folder


def pairs(
    folder: pathlib.Path, source: str, target: str, limit: int, seed: int
) -> list[tuple[str, str]]:
    left = next(folder.glob(f"*.{source}")).read_text(encoding="utf-8")
    right = next(folder.glob(f"*.{target}")).read_text(encoding="utf-8")
    rows = []
    seen = set()
    for first, second in zip(left.splitlines(), right.splitlines()):
        first = first.strip()
        second = second.strip()
        if not (MINIMUM <= len(first) <= MAXIMUM and MINIMUM <= len(second) <= MAXIMUM):
            continue
        if first in seen:
            continue
        seen.add(first)
        rows.append((first, second))
    random.Random(seed).shuffle(rows)
    return rows[:limit]


def run(corpus: str, source: str, target: str, limit: int, seed: int) -> dict:
    engine = translator(source, target)
    sample = pairs(prepare(corpus), source, target, limit, seed)
    hypotheses = []
    started = time.perf_counter()
    for text, _ in sample:
        hypotheses.append(engine(text))
    elapsed = time.perf_counter() - started
    references = [text for _, text in sample]
    chrf = CHRF().corpus_score(hypotheses, [references]).score
    bleu = BLEU().corpus_score(hypotheses, [references]).score
    return {
        "corpus": corpus,
        "pair": f"{source}-{target}",
        "segments": len(sample),
        "chrf": round(chrf, 2),
        "bleu": round(bleu, 2),
        "seconds": round(elapsed, 1),
        "seconds_per_segment": round(elapsed / len(sample), 3),
        "source_words": sum(len(text.split()) for text, _ in sample),
        "reference_words": sum(len(text.split()) for text in references),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--corpus", choices=sorted(CORPORA), default="tatoeba")
    parser.add_argument("--source", default="en")
    parser.add_argument("--target", default="pt")
    parser.add_argument("--limit", type=int, default=200)
    parser.add_argument("--seed", type=int, default=2026)
    parser.add_argument("--json", type=pathlib.Path)
    options = parser.parse_args()
    result = run(options.corpus, options.source, options.target, options.limit, options.seed)
    print(json.dumps(result, indent=2))
    if options.json:
        options.json.parent.mkdir(parents=True, exist_ok=True)
        options.json.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
