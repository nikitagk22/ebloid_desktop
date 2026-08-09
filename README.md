# Ebloid Desktop

[![Release](https://img.shields.io/github/v/release/nikitagk22/ebloid_desktop?label=version&color=007ec6)](https://github.com/nikitagk22/ebloid_desktop/releases)
[![License](https://img.shields.io/github/license/nikitagk22/ebloid_desktop?label=license&color=4c1)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white)](https://tauri.app/)
[![Stars](https://img.shields.io/github/stars/nikitagk22/ebloid_desktop?style=flat&logo=github)](https://github.com/nikitagk22/ebloid_desktop/stargazers)
[![Forks](https://img.shields.io/github/forks/nikitagk22/ebloid_desktop?style=flat&logo=github)](https://github.com/nikitagk22/ebloid_desktop/network/members)
[![Last Commit](https://img.shields.io/github/last-commit/nikitagk22/ebloid_desktop)](https://github.com/nikitagk22/ebloid_desktop/commits/main)
[![Issues](https://img.shields.io/github/issues/nikitagk22/ebloid_desktop)](https://github.com/nikitagk22/ebloid_desktop/issues)

Ebloid Desktop — приложение для [eblo.id](https://eblo.id/) на Windows,
macOS и Linux. Оно открывает сайт в системном WebView: интерфейс, лента,
аккаунт и публикации остаются на eblo.id, а приложение добавляет удобство
обычного desktop-клиента — отдельное окно, загрузку файлов, скачивания и
работу с системными уведомлениями.

## Скачать и установить

Готовые файлы находятся на странице
[Releases](https://github.com/nikitagk22/ebloid_desktop/releases).

Выберите только один файл для своей платформы:

| Устройство | Рекомендуемый файл |
| --- | --- |
| Обычный компьютер с Windows (Intel/AMD) | `Ebloid_*_x64-setup.exe` |
| Windows на ARM (например, Snapdragon) | `Ebloid_*_arm64-setup.exe` |
| Mac с Apple Silicon (M1/M2/M3/M4) | `Ebloid_*_aarch64.dmg` |
| Mac с процессором Intel | `Ebloid_*_x64.dmg` |
| Linux на Intel/AMD | `.AppImage`, `.deb` или `.rpm` с `amd64`/`x86_64` |
| Linux на ARM64 | `.AppImage`, `.deb` или `.rpm` с `arm64`/`aarch64` |

Для большинства пользователей Windows лучше выбрать файл `setup.exe`. В
Linux: `.deb` — Debian/Ubuntu, `.rpm` — Fedora/openSUSE, `.AppImage` —
переносимый вариант без установки.

На macOS перетащите Ebloid из открытого DMG в папку «Программы». Если macOS
покажет предупреждение о неизвестном разработчике, откройте «Системные
настройки → Конфиденциальность и безопасность» и подтвердите запуск. Полное
отсутствие такого предупреждения возможно только после Apple notarization;
для этого нужен сертификат Apple Developer у издателя.

## Приватность и данные

Клиент не содержит аналитики, рекламы, трекеров, собственного сервера или
кода для передачи ваших данных разработчику. Он не читает пароли, ключи
доступа (passkeys), файлы вне выбранных вами диалогов или cookie и не просит
присылать их в поддержку.

Авторизация, публикации, сообщения и другие данные обрабатываются сайтом
eblo.id и теми провайдерами, через которых вы входите (например, Twitch,
Telegram или Google), по их правилам. Приложение хранит локально обычный
профиль WebView: данные входа, cookies, localStorage/IndexedDB и кэш сайта.
Это нужно, чтобы не входить заново и не скачивать статику при каждом запуске.
Удаление приложения или очистка его данных в настройках ОС завершит локальную
сессию.

Не передавайте кому-либо cookies, пароли, одноразовые коды или файлы ключей —
для диагностики клиента они не нужны.

## Что работает

- постоянная авторизация и кэширование статики сайта;
- загрузка файлов, в том числе перетаскиванием в форму;
- скачивание файлов через системное окно выбора папки;
- Twitch и Telegram OAuth, если провайдер разрешает вход во встроенном WebView;
- отдельные окна, полноэкранное видео и Picture-in-Picture, если это
  поддерживает WebView вашей ОС и сам сайт;
- уведомления — после разрешения со стороны сайта и операционной системы;
- ссылки вида `ebloid://…`, которые открывают соответствующую страницу в уже
  запущенном приложении.

## Вход через Google и passkey

На экране входа «через Google» используется авторизация стороннего
провайдера. Google намеренно ограничивает OAuth и passkey во встроенных
WebView: это защита от подмены страницы входа. Поэтому ключ доступа, Touch ID,
Face ID или Windows Hello на таком шаге могут не завершить вход, хотя в
обычном Chrome/Safari всё работает.

Безопасное решение — запускать именно этот OAuth-шаг в системном браузере и
возвращать результат в приложение через `ebloid://` callback. Для этого нужен
доступ к настройкам OAuth и серверной части eblo.id/Twitch: callback должен
быть добавлен у провайдера и на сайте. Одной правкой desktop-клиента это
сделать нельзя: браузер получил бы сессию, но WebView приложения — нет.
Пока такой callback не настроен, используйте на странице входа «Другой
способ» (пароль, код или другой доступный провайдер).

## Для разработки

Проект построен на Tauri 2: WebView2 в Windows, WKWebView в macOS и
WebKitGTK в Linux. После установки Rust и Node.js LTS выполните `npm ci`,
затем `npm run dev`. Сборка: `npm run build`.

Workflow [`.github/workflows/build-desktop.yml`](.github/workflows/build-desktop.yml)
создаёт шесть нативных вариантов: Windows x64/ARM64, Linux x64/ARM64 и macOS
Intel/Apple Silicon. Тег вида `vX.Y.Z` создаёт GitHub Release.

Исходник иконки — `src-tauri/icons/icon.svg`. Скрипт
`scripts/regenerate_icons.py` создаёт прозрачную PNG-основу без лишних полей;
после него выполните `npm run tauri -- icon src-tauri/icons/icon.png`, чтобы
обновить `.icns`, `.ico` и PNG-наборы. Скрипту нужен отдельный tooling-venv с
`Pillow` и `CairoSVG`.

## Лицензия

Этот проект распространяется под лицензией [GNU General Public License v3.0 (GPLv3)](LICENSE).

