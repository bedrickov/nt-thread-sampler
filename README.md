# ThreadPulse

<div align="center">

![Platform](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20(x64)-0078D4?style=flat-square&logo=windows)
![Language](https://img.shields.io/badge/Language-Rust%202021-DEA584?style=flat-square&logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue?style=flat-square)
![Build](https://img.shields.io/badge/Build-Passing-brightgreen?style=flat-square)

**Низкоуровневый сэмплер потоков и времени исполнения Windows NT ядра.**  
*Высокоточный съём времени контекстных переходов, ядерного и пользовательского процессорного времени через нативный NT ToolHelp и QueryProcessTimes.*

</div>

---

## Архитектура

`ThreadPulse` разработан для профилирования и выявления «тяжёлых» фоновых потоков и скрытого CPU throttling без оверхеда WMI:
- Снапшоты процессов и потоков через нативный Win32 ToolHelp32 API (`CreateToolhelp32Snapshot`).
- Вычисление точного времени работы в пространстве ядра и пользователя (`GetProcessTimes`) с микросекундной дискретизацией (100-нс интервалы NT конвертируются в мкс).
- Нулевые внешние C-рантайм зависимости, компиляция в единый чистый бинарник.

## Сборка и тесты

```powershell
cargo build --release
cargo test
```

## Лицензия
MIT
