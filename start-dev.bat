@echo off
chcp 65001 >nul
rem ProjectLens 開発モード起動用バッチファイル
rem エクスプローラーからダブルクリックして実行してください。

title ProjectLens - 開発モード
cd /d "%~dp0"

echo ProjectLens を開発モードで起動しています...
echo 初回起動時は Rust のビルドに時間がかかります
echo.

call npm run tauri dev

if errorlevel 1 (
    echo.
    echo [エラー] ProjectLens の起動に失敗しました。
    pause
)
