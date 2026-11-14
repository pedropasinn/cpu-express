@echo off
setlocal
cd /d "%~dp0"
if not exist "%~dp0cpu-express.exe" goto sem_executavel
if not exist "%~dp0perfil-beta" mkdir "%~dp0perfil-beta"
if not exist "%~dp0diagnosticos" mkdir "%~dp0diagnosticos"
if not exist "%~dp0perfil-beta" goto sem_escrita
if not exist "%~dp0diagnosticos" goto sem_escrita
:novo_log
set "LOG=diagnostico-%RANDOM%-%RANDOM%.jsonl"
if exist "%~dp0diagnosticos\%LOG%" goto novo_log
echo CPU Express 0.4.0-rc.3 - teste com perfil separado nesta pasta.
echo Feche a outra sessao deste beta antes de continuar.
echo O diagnostico fica somente neste computador; nada e enviado automaticamente.
start "" /wait "%~dp0cpu-express.exe" --config "%~dp0perfil-beta\config.json" --diagnostics "%~dp0diagnosticos\%LOG%"
set "RESULT=%ERRORLEVEL%"
echo.
echo Codigo de saida: %RESULT%
echo Arquivo para enviar manualmente ao Pedro:
echo "%~dp0diagnosticos\%LOG%"
echo Ele inclui metricas, cidade e coordenadas configuradas para o clima.
pause
exit /b %RESULT%
:sem_executavel
echo Extraia o pacote completo e abra este arquivo ao lado de cpu-express.exe.
pause
exit /b 1
:sem_escrita
echo Extraia o pacote em uma pasta onde voce possa salvar arquivos, como Documentos.
pause
exit /b 1
