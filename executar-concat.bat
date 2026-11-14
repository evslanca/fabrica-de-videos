@echo off
rem Abre o Concat (build de debug ja compilado) com FFmpeg local ao projeto.
set "FFMPEG_DIR=%~dp0vendor\ffmpeg"
set "PATH=%FFMPEG_DIR%\bin;%PATH%"
cd /d "%~dp0src"
if exist "target\debug\concat.exe" (
  "target\debug\concat.exe"
) else (
  echo concat.exe nao encontrado. Compile antes com: cargo build -p concat
)
echo.
echo O app foi encerrado. Se houve erro, ele aparece acima.
pause
