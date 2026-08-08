# Ebloid Desktop

Нативное desktop-приложение для [eblo.id](https://eblo.id/), построенное как
удалённый WebView. На Windows используется Microsoft Edge WebView2; в macOS —
WKWebView; в Linux — WebKitGTK. Сайт остаётся единственным источником UI и
данных: приложение не подменяет его авторизацию и не получает учётные данные.

## Что уже поддерживает shell

- Twitch и Telegram OAuth, включая переходы между безопасными `https`-страницами;
- `window.open` и модальные окна, которые создаются как обычные дочерние окна;
- выбор одного и нескольких файлов через системный диалог;
- скачивания с обычным диалогом сохранения, который предоставляет runtime ОС;
- Picture-in-Picture и полноэкранное видео, если это поддерживает WebView runtime;
- разрешение на уведомления от `https://eblo.id` через настройки ОС/runtime.

Состояние авторизации хранится в отдельном профиле WebView. Поэтому после
перезапуска пользователь остаётся залогинен, пока не выйдет на самом сайте.

## Запуск на Windows

1. Установите [Rust](https://www.rust-lang.org/tools/install) и Node.js LTS.
2. Убедитесь, что установлен **Microsoft Edge WebView2 Runtime** (на актуальных
   Windows 10/11 обычно уже есть).
3. В этой папке выполните `npm install`, затем `npm run dev`.

Для установщика: `npm run build`. Готовый MSI/NSIS будет в
`src-tauri/target/release/bundle/`.

## Сборки в GitHub Actions

Workflow [`.github/workflows/build-desktop.yml`](.github/workflows/build-desktop.yml)
собирает шесть нативных вариантов: Windows x64/ARM64, Linux x64/ARM64 и macOS
Intel/Apple Silicon. После каждого push и pull request они прикрепляются к
запуску workflow как artifacts. Пуш тега вида `v0.1.0` дополнительно создаёт
GitHub Release с готовыми установщиками.

## Важное ограничение

Нативный shell может разрешить нативные возможности, но не может гарантировать
политику сторонних OAuth-провайдеров. Если Twitch или Telegram в будущем
запретят embedded-авторизацию, правильным продолжением будет callback-схема с
системным браузером и deep link — потребуется доступ к настройкам OAuth у сайта.
