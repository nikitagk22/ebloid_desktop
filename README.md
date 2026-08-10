<p align="center">
  <img src="src-tauri/icons/icon.svg" width="128" height="128" alt="Ebloid Logo" />
</p>

<h1 align="center">Ebloid Desktop</h1>

<p align="center">
  Неофициальный десктопный клиент eblo.id для Windows, macOS и Linux с одной кодовой базой и единым интерфейсом.
</p>

<p align="center">
  <a href="https://eblo.id/"><b>Открыть eblo.id</b></a>
</p>

---

<p align="center">
  <a href="https://github.com/nikitagk22/ebloid_desktop/releases"><img src="https://img.shields.io/github/v/release/nikitagk22/ebloid_desktop?label=version&color=007ec6" alt="Release" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/nikitagk22/ebloid_desktop?label=license&color=4c1" alt="License" /></a>
  <a href="https://tauri.app/"><img src="https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri&logoColor=white" alt="Tauri" /></a>
  <a href="https://github.com/nikitagk22/ebloid_desktop/stargazers"><img src="https://img.shields.io/github/stars/nikitagk22/ebloid_desktop?style=flat&logo=github" alt="Stars" /></a>
  <a href="https://github.com/nikitagk22/ebloid_desktop/network/members"><img src="https://img.shields.io/github/forks/nikitagk22/ebloid_desktop?style=flat&logo=github" alt="Forks" /></a>
  <a href="https://github.com/nikitagk22/ebloid_desktop/commits/main"><img src="https://img.shields.io/github/last-commit/nikitagk22/ebloid_desktop" alt="Last Commit" /></a>
  <a href="https://github.com/nikitagk22/ebloid_desktop/issues"><img src="https://img.shields.io/github/issues/nikitagk22/ebloid_desktop" alt="Issues" /></a>
</p>

Ebloid Desktop — неофициальное приложение для [eblo.id](https://eblo.id/) на Windows,
macOS и Linux. Оно открывает сайт в системном WebView: интерфейс, лента,
аккаунт и публикации остаются на eblo.id, а приложение добавляет удобство
обычного desktop-клиента — отдельное окно, загрузку файлов, скачивания и
работу с системными уведомлениями.

## Скачать и установить

Готовые файлы находятся на странице
[Releases](https://github.com/nikitagk22/ebloid_desktop/releases).
Описание изменений каждой версии находится в
[CHANGELOG.md](CHANGELOG.md).

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
кода для передачи ваших данных разработчику. Он не читает пароли и ключи
доступа (passkeys), не отправляет разработчику cookies и не просит присылать
их в поддержку. Приложение получает доступ только к файлам, которые вы сами
выбрали или перетащили в окно сайта.

Авторизация, публикации, сообщения и другие данные обрабатываются сайтом
eblo.id и теми провайдерами, через которых вы входите (например, Twitch,
Telegram или Google), по их правилам. Приложение хранит локально обычный
профиль WebView: данные входа, cookies, localStorage/IndexedDB и кэш сайта.
Это нужно, чтобы не входить заново и не скачивать статику при каждом запуске.
Встроенный наблюдатель читает только уже показанные на странице элементы
уведомлений: их идентификатор, тип, текст, ссылку и число непрочитанных. Это
нужно для системных уведомлений и счётчика на иконке; сведения остаются на
компьютере и никуда дополнительно не отправляются. Удаление приложения,
кнопка выхода в настройках клиента или очистка его данных в настройках ОС
завершит локальную сессию.

Не передавайте кому-либо cookies, пароли, одноразовые коды или файлы ключей —
для диагностики клиента они не нужны.

## Что работает

- постоянная авторизация и кэширование статики сайта;
- загрузка файлов, в том числе перетаскиванием в форму;
- менеджер загрузок с автоматическим сохранением в системную папку «Загрузки»,
  прогрессом, отменой, историей и кнопкой «Показать в папке»;
- отдельная страница настроек: уведомления, автозапуск, трей, масштаб,
  очистка кэша и полный выход с удалением cookies;
- нативные уведомления о комментариях, ответах, оценках, видео и системных
  событиях, а также счётчик непрочитанных в Dock/панели задач и трее;
- сворачивание в трей и восстановление размера/позиции окна;
- тематический экран отсутствия сети с повторным подключением и watchdog,
  перезагружающим зависший WebView без закрытия всего приложения;
- контекстное меню ссылок и изображений: внешний браузер, копирование ссылки
  и сохранение изображения;
- вставка изображений из буфера, HTML5 drag-and-drop и управление видео
  системными медиаклавишами через Media Session;
- загрузка файлов в публикацию через обычный выбор, перетаскивание или вставку
  изображения из буфера — без изменения системных ассоциаций файлов;
- проверка и установка криптографически подписанных обновлений из GitHub
  Releases внутри приложения;
- Twitch и Telegram OAuth, если провайдер разрешает вход во встроенном WebView;
- отдельные окна, полноэкранное видео и Picture-in-Picture, если это
  поддерживает WebView вашей ОС и сам сайт;
- уведомления — после разрешения со стороны сайта и операционной системы;
- ссылки вида `ebloid://…`, которые открывают соответствующую страницу в уже
  запущенном приложении.

Горячие клавиши: `Ctrl/Cmd + ,` открывает настройки клиента,
`Ctrl/Cmd + Shift + D` — менеджер загрузок. Масштаб сайта также меняется
обычными `Ctrl/Cmd + +`, `Ctrl/Cmd + -` и `Ctrl/Cmd + 0`.

Файлы до лимита самого сайта (сейчас форма указывает 200 МБ) можно загрузить
обычным перетаскиванием или кнопкой выбора файла.

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
Intel/Apple Silicon. Тег вида `vX.Y.Z` создаёт GitHub Release, подписанные
updater-пакеты и общий `latest.json`. Приватный ключ хранится только в GitHub
Secret `TAURI_SIGNING_PRIVATE_KEY`; в репозитории находится публичный ключ.

Исходник иконки — `src-tauri/icons/icon.svg`. Скрипт
`scripts/regenerate_icons.py` создаёт прозрачную PNG-основу без лишних полей;
после него выполните `npm run tauri -- icon src-tauri/icons/icon.png`, чтобы
обновить `.icns`, `.ico` и PNG-наборы. Скрипту нужен отдельный tooling-venv с
`Pillow` и `CairoSVG`.

Отдельный `scripts/generate_badges.py` пересоздаёт PNG-счётчики 1–9+ для
overlay-иконки панели задач Windows и требует только `Pillow`.

## Лицензия

Этот проект распространяется под лицензией [GNU General Public License v3.0 (GPLv3)](LICENSE).
