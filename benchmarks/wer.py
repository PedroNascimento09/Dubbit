import argparse
import json
import pathlib
import re
import sys
import unicodedata

WORD = re.compile(r"[a-z0-9']+")


def normalize(text):
    lowered = unicodedata.normalize("NFKC", text).lower()
    lowered = lowered.replace("’", "'")
    return WORD.findall(lowered)


def srt_words(path):
    blocks = pathlib.Path(path).read_text(encoding="utf-8").strip().split("\n\n")
    lines = []
    for block in blocks:
        rows = [row for row in block.splitlines() if row.strip()]
        if len(rows) < 3:
            continue
        text = " ".join(rows[2:])
        if "-->" in text:
            continue
        lines.append(text)
    return normalize(" ".join(lines))


def align(reference, hypothesis):
    rows = len(reference) + 1
    columns = len(hypothesis) + 1
    distance = [[0] * columns for _ in range(rows)]
    for row in range(rows):
        distance[row][0] = row
    for column in range(columns):
        distance[0][column] = column
    for row in range(1, rows):
        for column in range(1, columns):
            cost = 0 if reference[row - 1] == hypothesis[column - 1] else 1
            distance[row][column] = min(
                distance[row - 1][column] + 1,
                distance[row][column - 1] + 1,
                distance[row - 1][column - 1] + cost,
            )
    counts = {"substitutions": 0, "deletions": 0, "insertions": 0, "matches": 0}
    row = rows - 1
    column = columns - 1
    while row > 0 or column > 0:
        if row > 0 and column > 0:
            cost = 0 if reference[row - 1] == hypothesis[column - 1] else 1
            if distance[row][column] == distance[row - 1][column - 1] + cost:
                counts["matches" if cost == 0 else "substitutions"] += 1
                row -= 1
                column -= 1
                continue
        if row > 0 and distance[row][column] == distance[row - 1][column] + 1:
            counts["deletions"] += 1
            row -= 1
            continue
        counts["insertions"] += 1
        column -= 1
    return counts


def score(reference, hypothesis):
    counts = align(reference, hypothesis)
    errors = counts["substitutions"] + counts["deletions"] + counts["insertions"]
    total = len(reference)
    return {
        "reference_words": total,
        "hypothesis_words": len(hypothesis),
        "errors": errors,
        "wer": round(errors / total, 4) if total else None,
        "substitutions": counts["substitutions"],
        "deletions": counts["deletions"],
        "insertions": counts["insertions"],
    }


def main():
    parser = argparse.ArgumentParser(
        description="Compute word error rate for a subtitle file against a reference transcript."
    )
    parser.add_argument("--reference", required=True, help="Plain text reference transcript")
    parser.add_argument("--srt", required=True, help="Subtitle file produced by the pipeline")
    parser.add_argument("--label", default=None, help="Name included in the output")
    arguments = parser.parse_args()

    reference = normalize(pathlib.Path(arguments.reference).read_text(encoding="utf-8"))
    hypothesis = srt_words(arguments.srt)
    result = score(reference, hypothesis)
    if arguments.label:
        result["label"] = arguments.label

    print(json.dumps(result, indent=2))
    if result["wer"] is None:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
