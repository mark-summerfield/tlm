SET CARGO_TARGET_DIR=target
SET RELEASE_DIR=%CARGO_TARGET_DIR%\release
cargo build --release -- %*
copy /Y target\release\musicbox.exe .
rcedit musicbox.exe --set-icon images\musicbox.ico
copy /Y musicbox.exe C:\bin
