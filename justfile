# Bascan — Local Manga Reader

set shell := ["pwsh", "-NoProfile", "-Command"]

# Default library path (override with: just run C:\path\to\library)
library := "library"

# Scheduled task name used by autostart/update/status/logs
task_name := "Bascan"

# Resolved at recipe-eval time so absolute paths are stable across cwd
exe_path := justfile_directory() / "packages" / "backend-rs" / "target" / "release" / "bascan-backend.exe"
log_dir  := "$env:LOCALAPPDATA\\Bascan"
log_file := "$env:LOCALAPPDATA\\Bascan\\bascan.log"

# Build everything: frontend + Rust binary
build:
    pnpm --prefix packages/frontend build; cargo build --release --manifest-path packages/backend-rs/Cargo.toml

# Run the production binary (optionally pass library path)
run path=library:
    packages/backend-rs/target/release/bascan-backend.exe {{path}}

# Dev mode: Rust backend + Vite frontend (with proxy)
dev path=library:
    Start-Process -NoNewWindow -FilePath "cargo" -ArgumentList "run","--manifest-path","packages/backend-rs/Cargo.toml","--","{{path}}"; pnpm --prefix packages/frontend dev

# Install frontend dependencies
install:
    pnpm --prefix packages/frontend install

# Type-check frontend
check:
    pnpm --prefix packages/frontend check

# ──────────────────────────────────────────────────────────────────────────────
# Autostart (Windows Scheduled Task, user scope, hidden, at logon)
# ──────────────────────────────────────────────────────────────────────────────

# Register/refresh the "Bascan" scheduled task. Runs hidden at logon as current user.
# Uses a wscript+vbs launcher so the console window is actually invisible
# (the task scheduler's "Hidden" flag does NOT hide console windows).
# Usage: just install-autostart                       (uses ./library)
#        just install-autostart C:\path\to\library
install-autostart path=library:
    $exe = ('{{exe_path}}' -replace '/', '\'); \
    if (-not (Test-Path $exe)) { Write-Error 'Binary not found. Run `just build` first.'; exit 1 }; \
    $lib = (Resolve-Path '{{path}}').Path; \
    New-Item -ItemType Directory -Force -Path {{log_dir}} | Out-Null; \
    $log     = Join-Path $env:LOCALAPPDATA 'Bascan\bascan.log'; \
    $cmdFile = Join-Path $env:LOCALAPPDATA 'Bascan\launch.cmd'; \
    $vbsFile = Join-Path $env:LOCALAPPDATA 'Bascan\launch.vbs'; \
    $cmdBody = "@echo off`r`ncd /d `"$(Split-Path $exe -Parent)`"`r`n`"$exe`" `"$lib`" >> `"$log`" 2>&1`r`n"; \
    Set-Content -LiteralPath $cmdFile -Value $cmdBody -Encoding ASCII -NoNewline; \
    $vbsBody = "Set s = CreateObject(`"WScript.Shell`")`r`ns.Run `"`"`"" + $cmdFile + "`"`"`", 0, False`r`n"; \
    Set-Content -LiteralPath $vbsFile -Value $vbsBody -Encoding ASCII -NoNewline; \
    $action    = New-ScheduledTaskAction    -Execute 'wscript.exe' -Argument ('"' + $vbsFile + '"'); \
    $trigger   = New-ScheduledTaskTrigger   -AtLogOn -User $env:USERNAME; \
    $settings  = New-ScheduledTaskSettingsSet -Hidden -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1); \
    $principal = New-ScheduledTaskPrincipal -UserId $env:USERNAME -LogonType Interactive; \
    if (Get-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue) { Unregister-ScheduledTask -TaskName '{{task_name}}' -Confirm:$false }; \
    Register-ScheduledTask -TaskName '{{task_name}}' -Action $action -Trigger $trigger -Settings $settings -Principal $principal | Out-Null; \
    Start-ScheduledTask -TaskName '{{task_name}}'; \
    Write-Host "Installed and started task '{{task_name}}'. Library: $lib"; \
    Write-Host "Launcher: $vbsFile -> $cmdFile"; \
    Write-Host "Logs    : $log"

# Remove the autostart task and stop any running instance
uninstall-autostart:
    if (Get-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue) { \
        Stop-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue; \
        Unregister-ScheduledTask -TaskName '{{task_name}}' -Confirm:$false; \
        Write-Host "Removed task '{{task_name}}'."; \
    } else { Write-Host "Task '{{task_name}}' not registered."; }; \
    Get-Process -Name 'bascan-backend' -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue; \
    Write-Host "Stopped any running bascan-backend processes."

# Rebuild and restart the autostart task. The exe is locked while running,
# so we kill the process first, then build, then start the task again.
update path=library:
    Write-Host '[1/3] Stopping running instance...'; \
    if (Get-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue) { Stop-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue }; \
    Get-Process -Name 'bascan-backend' -ErrorAction SilentlyContinue | Stop-Process -Force; \
    Start-Sleep -Milliseconds 500; \
    Write-Host '[2/3] Building...'; \
    pnpm --prefix packages/frontend build; \
    cargo build --release --manifest-path packages/backend-rs/Cargo.toml; \
    if ($LASTEXITCODE -ne 0) { Write-Error 'Build failed.'; exit 1 }; \
    Write-Host '[3/3] Starting task...'; \
    if (Get-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue) { \
        Start-ScheduledTask -TaskName '{{task_name}}'; \
        Write-Host "Update complete. http://localhost:3001"; \
    } else { Write-Host "Task not installed. Run `just install-autostart {{path}}` to register it."; }

# Tail the autostart log
logs:
    if (-not (Test-Path {{log_file}})) { Write-Host 'No log file yet at {{log_file}}'; exit 0 }; \
    Get-Content -Path {{log_file}} -Wait -Tail 50

# Show task + process status
status:
    $t = Get-ScheduledTask -TaskName '{{task_name}}' -ErrorAction SilentlyContinue; \
    if ($t) { \
        $info = $t | Get-ScheduledTaskInfo; \
        Write-Host ("Task        : {0}" -f $t.State); \
        Write-Host ("Last run    : {0}" -f $info.LastRunTime); \
        Write-Host ("Last result : 0x{0:X}" -f $info.LastTaskResult); \
    } else { Write-Host 'Task        : not installed'; }; \
    $p = Get-Process -Name 'bascan-backend' -ErrorAction SilentlyContinue; \
    if ($p) { \
        Write-Host ("Process     : running (PID {0}, {1:N1} MB)" -f $p.Id, ($p.WorkingSet64/1MB)); \
        Write-Host  "URL         : http://localhost:3001"; \
    } else { Write-Host 'Process     : not running'; }
