# use PowerShell instead of sh:
set shell := ["pwsh.exe", "-NoProfile",  "-c"]

setup:
    just pre-commit

remove-pre-commit:
    pre-commit uninstall
    pre-commit uninstall --hook-type commit-msg
    pre-commit uninstall --hook-type pre-push

pre-commit:
    pre-commit install
    pre-commit install --hook-type commit-msg
    pre-commit install --hook-type pre-push

check:
    cargo c

build-all:
    just build x86_64
    just build i686
    just build aarch64

build TARGET:
    cargo b -r --target {{ TARGET }}-pc-windows-msvc

beta-build TARGET:
    cargo b -r --target {{ TARGET }}-pc-windows-msvc --features beta

clean:
    if (Test-Path "release") { rm -r "release" -Force -ErrorAction Ignore }
    mkdir "release"

release TARGET:
    just build {{ TARGET }}

    cp ./target/{{ TARGET }}-pc-windows-msvc/release/sfsu.exe ./release/sfsu.exe
    7z a ./release/dl-{{ TARGET }} ./release/sfsu.exe
    mv ./release/sfsu.exe ./release/sfsu-{{ TARGET }}.exe
    just export-hash {{ TARGET }}

beta-release TARGET:
    just beta-build {{ TARGET }}

    cp ./target/{{ TARGET }}-pc-windows-msvc/release/sfsu.exe ./release/sfsu.exe
    7z a ./release/dl-{{ TARGET }} ./release/sfsu.exe
    mv ./release/sfsu.exe ./release/sfsu-{{ TARGET }}.exe
    just export-hash {{ TARGET }}

release-all: clean
    just release x86_64
    just release i686
    just release aarch64

beta-release-all: clean
    just beta-release x86_64
    just beta-release i686
    just beta-release aarch64

export-hash TARGET:
    python scripts/hash.py './release/dl-{{ TARGET }}.7z'
    python scripts/hash.py './release/sfsu-{{ TARGET }}.exe'
