SET CARGO_TARGET_DIR=target
SET RELEASE_DIR=%CARGO_TARGET_DIR%\release
cargo build --release -- %*
copy /Y target\release\tlm.exe .
rcedit tlm.exe --set-icon images\tlm.ico
copy /Y tlm.exe C:\bin
