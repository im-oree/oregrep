$root = $PSScriptRoot
$output = Join-Path $root "project_files.txt"

# Directory names to skip entirely (anywhere in the tree)
$excludeDirs = @(
    'node_modules', 'dist', 'release', 'target', '.git', 'build',
    'win-unpacked', '.vscode', '.idea', 'coverage', 'out', 'pkg'
)

# File name patterns to skip
$excludeFilePatterns = @(
    '*.bak*', '*.db', '*.db-shm', '*.db-wal',
    '*.wasm', '*.wasm.d.ts', '*.exe', '*.dll', '*.pdb', '*.lib', '*.exp',
    '*.blockmap', '*.map', '*.pak', '*.dat', '*.bin',
    '*.rlib', '*.rmeta', '*.d', '*.json.timestamp',
    'package-lock.json', '*.lock', 'invoked.timestamp',
    '*.zip', '*.onion'
)

function Test-ExcludedFile {
    param([string]$Name)
    foreach ($pattern in $excludeFilePatterns) {
        if ($Name -like $pattern) { return $true }
    }
    return $false
}

function Get-FilesFiltered {
    param([string]$Path)

    Get-ChildItem -LiteralPath $Path -File | ForEach-Object {
        if (-not (Test-ExcludedFile $_.Name)) {
            $_.FullName.Substring($root.Length + 1)
        }
    }

    Get-ChildItem -LiteralPath $Path -Directory |
        Where-Object { $excludeDirs -notcontains $_.Name } |
        ForEach-Object {
            Get-FilesFiltered -Path $_.FullName
        }
}

Get-FilesFiltered -Path $root |
    Sort-Object |
    Set-Content -Path $output -Encoding UTF8

notepad $output