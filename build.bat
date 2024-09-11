cargo build --release

powershell Compress-Archive -DestinationPath build.zip -Force -Path target\release\funge-it-together.exe, README.md, LICENSE, levels