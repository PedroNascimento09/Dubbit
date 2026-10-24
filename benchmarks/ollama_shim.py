#!/usr/bin/env python3
import argparse
import json
import re
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import ClassVar

from argos_engine import translator

NAMES = {
    "English": "en",
    "Portuguese": "pt",
    "Spanish": "es",
    "French": "fr",
    "German": "de",
    "Italian": "it",
}

PATTERN = re.compile(r"from (\w+) to (\w+)")


class Handler(BaseHTTPRequestHandler):
    engines: ClassVar[dict] = {}

    def do_POST(self) -> None:
        length = int(self.headers.get("Content-Length", "0"))
        body = json.loads(self.rfile.read(length) or b"{}")
        prompt = body.get("prompt", "")
        found = PATTERN.search(prompt)
        text = prompt.rsplit("\n\n", 1)[-1].strip()
        answer = ""
        if found and text:
            source = NAMES.get(found.group(1), found.group(1).lower()[:2])
            target = NAMES.get(found.group(2), found.group(2).lower()[:2])
            key = (source, target)
            if key not in self.engines:
                self.engines[key] = translator(source, target)
            answer = self.engines[key](text)
        payload = json.dumps({"response": answer}).encode("utf-8")
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(payload)))
        self.end_headers()
        self.wfile.write(payload)

    def log_message(self, *args) -> None:
        pass


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=11500)
    options = parser.parse_args()
    server = HTTPServer(("127.0.0.1", options.port), Handler)
    print(f"listening on http://127.0.0.1:{options.port}", flush=True)
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
