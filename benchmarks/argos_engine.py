#!/usr/bin/env python3
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent


class Line:
    def split_sentences(self, text: str) -> list[str]:
        return [text]


def translator(source: str, target: str):
    import ctranslate2
    from argostranslate import package, translate

    installed = package.get_installed_packages()
    chosen = next(
        item for item in installed if item.from_code == source and item.to_code == target
    )
    model = chosen.package_path / "model"
    runtime = ctranslate2.Translator(str(model), device="cpu", compute_type="int8")
    shim = Line()

    def one(text: str) -> str:
        translated = translate.apply_packaged_translation(chosen, text, runtime, shim, 1)
        return translated[0].value.strip()

    return one
