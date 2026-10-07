#!/usr/bin/env python3
"""Create a new disposable desktop acceptance project through the real headless API.

No existing destination is replaced. Project schema and validation remain core-owned.
"""
import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import wave


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--headless", type=Path, required=True)
    args = parser.parse_args()
    root = args.destination.resolve()
    root.mkdir(parents=True, exist_ok=False)
    for name in ("projects", "media", "exports"):
        (root / name).mkdir()
    env = dict(os.environ, OPENCUT_PROJECTS_DIR=str(root / "projects"),
               OPENCUT_ALLOWED_MEDIA_DIRS=str(root / "media"), OPENCUT_EXPORTS_DIR=str(root / "exports"))
    binary = args.headless.resolve()
    catalog = json.loads((Path(__file__).resolve().parents[1] / "contracts/desktop-compositing-controls-v1.json").read_text())
    requests = []

    def request(value):
        result = subprocess.run([str(binary)], input=json.dumps(value), text=True,
                                capture_output=True, env=env, check=False)
        reply = json.loads(result.stdout)
        requests.append({"request": value, "response": reply})
        if result.returncode or reply.get("type") != "result":
            raise RuntimeError(f"Core rejected fixture request: {reply}")
        return reply["result"]

    project_id = request({"operation": "create_project", "name": "Desktop compositing acceptance",
                          "settings": {"width": 64, "height": 64, "fps": 10}})["projectId"]
    def state():
        return request({"operation": "get_state", "projectId": project_id})["project"]
    def edit(value):
        return request({"operation": "edit", "projectId": project_id,
                        "expectedRevision": state()["revision"], "edit": value})
    track = state()["tracks"][1]["id"]
    aliases = {}
    def add(name, value):
        result = edit(value)
        aliases[name] = result["changedIds"][0]
        return aliases[name]
    rectangle = {"operation": "add_rectangle", "trackId": track, "startMs": 0, "durationMs": 1000,
                 "width": 64, "height": 64, "color": "#ffffff",
                 "transform": {"positionX": 0, "positionY": 0, "scale": 1, "opacity": 1}}
    provider = add("provider", rectangle)
    edit({"operation": "update_item", "itemId": provider, "matteOnly": True})
    leaf = add("leaf", rectangle)
    mask = copy.deepcopy(catalog["defaultMask"])
    mask["source"]["path"]["commands"] = [
        {"type": "moveTo", "to": {"x": 0, "y": 0}},
        {"type": "lineTo", "to": {"x": 64, "y": 0}},
        {"type": "quadraticTo", "control": {"x": 64, "y": 32}, "to": {"x": 64, "y": 64}},
        {"type": "cubicTo", "control1": {"x": 48, "y": 64}, "control2": {"x": 16, "y": 64}, "to": {"x": 0, "y": 64}},
        {"type": "close"}]
    gradient = copy.deepcopy(catalog["defaultMask"])
    gradient["id"] = "gradient"
    gradient["source"]["paint"] = {"type": "linearGradient", "start": {"x": 0, "y": 0},
                                    "end": {"x": 64, "y": 64}, "stops": [
                                        {"offset": 0, "color": {"r": 1, "g": 0, "b": 0, "a": 0.25}},
                                        {"offset": 1, "color": {"r": 0, "g": 1, "b": 0, "a": 0.75}}]}
    edit({"operation": "update_item", "itemId": leaf, "masks": [mask, gradient],
          "effects": [catalog["effectDefaults"]["vignette"]]})
    edit({"operation": "set_animation_channels", "itemId": leaf, "animationChannels": [
        {"property": "mask.transform.opacity", "target": {"kind": "mask", "scope": "root", "id": "mask-1"},
         "keyframes": [{"timeMs": 0, "value": {"type": "scalar", "value": 1}, "curve": "linear"}]},
        {"property": "effect.vignette_amount", "target": {"kind": "effect", "scope": "root", "id": "effect-1"},
         "keyframes": [{"timeMs": 0, "value": {"type": "scalar", "value": 0.5}, "curve": "linear"}]}]})
    add("group", {"operation": "add_group", "trackId": track, "startMs": 0, "durationMs": 1000})
    local = {"type": "rectangle", "id": "local-leaf", "width": 32, "height": 32, "color": "#ffffff",
             "startMs": 0, "durationMs": 1000, "keyframes": [], "masks": [catalog["defaultMask"]]}
    component = add("component", {"operation": "component_create", "name": "Read-only local controls",
                                 "width": 64, "height": 64, "durationMs": 1000,
                                 "tracks": [{"id": "local", "name": "Local", "trackType": "overlay", "items": [local]}]})
    add("instance", {"operation": "add_component_instance", "trackId": track, "componentId": component,
                     "startMs": 0, "trimStartMs": 0, "durationMs": 1000, "timeScale": 1})
    audio = root / "media" / "silence.wav"
    with wave.open(str(audio), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(48000)
        wav.writeframes(b"\0\0" * 48000)
    asset = request({"operation": "import_asset", "projectId": project_id,
                     "expectedRevision": state()["revision"], "path": str(audio), "mediaType": "audio"})["changedIds"][0]
    add("audio", {"operation": "add_media", "trackId": state()["tracks"][2]["id"], "assetId": asset,
                  "startMs": 0, "durationMs": 1000, "sourceInMs": 0})
    project = state()
    receipt = {"projectId": project_id, "projectStore": str(root / "projects"), "aliases": aliases,
               "initialProject": project, "requests": requests,
               "headlessSha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
    (root / "fixture-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps({k: receipt[k] for k in ("projectId", "projectStore", "aliases")}, indent=2))


if __name__ == "__main__":
    main()
