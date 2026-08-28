param(
    [string]$Python = "python",
    [string]$DataRoot = ""
)

$ErrorActionPreference = "Stop"
$appRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = Resolve-Path (Join-Path $appRoot "..\..")
if (-not $DataRoot) {
    $DataRoot = Join-Path $repoRoot "local-data\kokoro"
}
$modelRoot = Join-Path $DataRoot "model"
$workRoot = Join-Path $DataRoot "work"
$venvRoot = Join-Path $DataRoot "venv"
$venvPython = Join-Path $venvRoot "Scripts\python.exe"

New-Item -ItemType Directory -Force -Path $modelRoot, $workRoot | Out-Null
if (-not (Test-Path -LiteralPath $venvPython)) {
    & $Python -m venv $venvRoot
}

& $venvPython -m pip install --upgrade "pip==25.1.1"
& $venvPython -m pip install "torch==2.8.0" --index-url https://download.pytorch.org/whl/cpu
& $venvPython -m pip install -r (Join-Path $appRoot "requirements-cpu.lock")

$env:OPENCUT_KOKORO_MODEL_DIR = $modelRoot
$env:OPENCUT_TTS_WORK_DIR = $workRoot
$env:HF_HOME = $modelRoot
$env:CUDA_VISIBLE_DEVICES = ""
Remove-Item Env:HF_HUB_OFFLINE -ErrorAction SilentlyContinue
& $venvPython (Join-Path $appRoot "worker.py") --prepare

Write-Host ""
Write-Host "Add these values to the OpenCut MCP environment:"
Write-Host "OPENCUT_KOKORO_PYTHON=$venvPython"
Write-Host "OPENCUT_KOKORO_MODEL_DIR=$modelRoot"
Write-Host "OPENCUT_TTS_WORK_DIR=$workRoot"
