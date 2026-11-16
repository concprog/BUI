# Installs `bui`, the command that makes, runs, checks and updates BUI apps.
#
#   irm https://bui.preetham.top/install.ps1 | iex
#
# Windows (x86_64 and arm64), Windows PowerShell 5.1 and PowerShell 7. It downloads the newest release
# from GitHub, checks its checksum, puts bui.exe in %USERPROFILE%\.bui\bin and adds that folder to your
# user PATH.
#
#   $env:BUI_VERSION = 'v0.5.1'     install that release instead of the newest
#   $env:BUI_INSTALL_DIR = 'DIR'    install into DIR
#   $env:BUI_NO_MODIFY_PATH = '1'   do not change PATH
#
# SPDX-License-Identifier: GPL-2.0-or-later

& {
    $ErrorActionPreference = 'Stop'
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    $ProgressPreference = 'SilentlyContinue' # Windows PowerShell 5.1 crawls with its own progress bar

    $repo = 'ppmpreetham/BUI'
    $installDir = if ($env:BUI_INSTALL_DIR) { $env:BUI_INSTALL_DIR } else { Join-Path $env:USERPROFILE '.bui\bin' }
    $baseUrl = $env:BUI_INSTALLER_BASE_URL # for testing: a folder that holds the archive and its .sha256

    $esc = [char]27
    $color = [Environment]::UserInteractive -and -not $env:NO_COLOR -and ($Host.UI.SupportsVirtualTerminal -or $env:WT_SESSION)
    function C($code, $text) { if ($color) { "$esc[${code}m$text$esc[0m" } else { $text } }
    # the glyphs are made from character codes: this file stays plain ASCII, which any PowerShell
    # reads the same whatever encoding it downloads it with
    $dot = [char]0x25CF; $tick = [char]0x2714; $cross = [char]0x2718
    function Glyphs($art) { # @ = full block, 1 2 3 4 5 6 = the box corners and lines of the logo
        $art.Replace('@', [string][char]0x2588).Replace('1', [string][char]0x2557).Replace('2', [string][char]0x2554).Replace('3', [string][char]0x255D).Replace('4', [string][char]0x255A).Replace('5', [string][char]0x2551).Replace('6', [string][char]0x2550)
    }
    function Step($text) { Write-Host "$(C '38;5;208' $dot) $text" }
    function Ok($text) { Write-Host "$(C '32' $tick) $text" }
    function Fail($text) { Write-Host "$(C '31' "$cross $text")"; throw $text }

    Write-Host ''
    Write-Host (C '38;5;208' (Glyphs '@@@@@@1 @@1   @@1@@1'))
    Write-Host (C '38;5;208' (Glyphs '@@266@@1@@5   @@5@@5'))
    Write-Host (C '38;5;208' (Glyphs '@@@@@@23@@5   @@5@@5'))
    Write-Host (C '38;5;208' (Glyphs '@@266@@1@@5   @@5@@5'))
    Write-Host "$(C '38;5;208' (Glyphs '@@@@@@234@@@@@@23@@5'))   $(C '1' 'installing bui')"
    Write-Host (C '38;5;208' (Glyphs '4666663  4666663 463'))
    Write-Host ''

    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    $triple = switch ($arch) {
        'X64' { 'x86_64-pc-windows-msvc' }
        'Arm64' { 'aarch64-pc-windows-msvc' }
        default { Fail "there is no bui build for the $arch processor" }
    }

    $tag = $env:BUI_VERSION
    if (-not $tag) {
        Step 'Looking for the newest release'
        # the tag comes from the page's redirect (the API would rate-limit)
        $location = $null
        try {
            $r = Invoke-WebRequest -Uri "https://github.com/$repo/releases/latest" -MaximumRedirection 0 -UseBasicParsing -ErrorAction Stop
            $location = $r.Headers['Location']
        } catch {
            if ($_.Exception.Response -and $_.Exception.Response.Headers) { $location = [string]$_.Exception.Response.Headers['Location'] }
        }
        if ($location -notmatch '/tag/(v\d[^/]*)$') { Fail "could not find the newest release of $repo (offline, or no release yet)" }
        $tag = $Matches[1]
    }
    if ($tag -notlike 'v*') { $tag = "v$tag" }

    $asset = "bui-$triple.zip"
    $url = if ($baseUrl) { "$baseUrl/$asset" } else { "https://github.com/$repo/releases/download/$tag/$asset" }
    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("bui-install-" + [guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp | Out-Null
    try {
        Step "Downloading bui $($tag.TrimStart('v')) for $triple"
        Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmp $asset) -UseBasicParsing
        Invoke-WebRequest -Uri "$url.sha256" -OutFile (Join-Path $tmp "$asset.sha256") -UseBasicParsing
        $want = ((Get-Content (Join-Path $tmp "$asset.sha256") -Raw).Trim() -split '\s+')[0].ToLower()
        $got = (Get-FileHash (Join-Path $tmp $asset) -Algorithm SHA256).Hash.ToLower()
        if ($want -ne $got) { Fail 'the download does not match its checksum: not installing it' }
        Ok 'Checksum matches'

        Expand-Archive -Path (Join-Path $tmp $asset) -DestinationPath $tmp -Force
        if (-not (Test-Path (Join-Path $tmp 'bui.exe'))) { Fail 'the download has no bui.exe in it' }
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
        # a running bui cannot be overwritten, but it can be renamed
        $target = Join-Path $installDir 'bui.exe'
        if (Test-Path $target) { Move-Item $target "$target.old" -Force }
        Copy-Item (Join-Path $tmp 'bui.exe') $target -Force
        Remove-Item "$target.old" -Force -ErrorAction SilentlyContinue
        Ok "Installed $target"
    } finally {
        Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $onPath = ($userPath -split ';') -contains $installDir
    if (-not $onPath -and -not $env:BUI_NO_MODIFY_PATH) {
        [Environment]::SetEnvironmentVariable('Path', "$installDir;$userPath", 'User')
        $env:Path = "$installDir;$env:Path"
        Ok "Added $installDir to your PATH"
    }

    Write-Host ''
    Write-Host "$(C '1;32' 'bui is ready.') Open a new terminal, then:"
    Write-Host ''
    Write-Host "  $(C '38;5;208' 'bui new my-app')   $(C '2' 'make an app')"
    Write-Host "  $(C '38;5;208' 'bui doctor')       $(C '2' 'check your setup')"
    Write-Host "  $(C '38;5;208' 'bui update')       $(C '2' 'get newer versions later')"
    Write-Host ''
}
