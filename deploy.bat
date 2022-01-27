SET CARGO_TARGET_DIR=target
SET RELEASE_DIR=%CARGO_TARGET_DIR%\release
cargo build --release -- %*
copy /Y target\release\mlm.exe .
rcedit mlm.exe --set-icon images\mlm.ico
copy /Y mlm.exe C:\bin
