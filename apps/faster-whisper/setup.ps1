$ErrorActionPreference = "Stop"
$root = Resolve-Path (Join-Path $PSScriptRoot "../..")
$data = Join-Path $root "local-data/transcription"
$venv = Join-Path $data "venv"
py -3.11 -m venv $venv
& (Join-Path $venv "Scripts/python.exe") -m pip install -r (Join-Path $PSScriptRoot "requirements-cpu.lock")
& (Join-Path $venv "Scripts/python.exe") (Join-Path $PSScriptRoot "worker.py") --prepare --model-dir (Join-Path $data "model")
