# ⌨️ TUI Typer

A clean, minimalistic, and high-performance terminal typing speed test built in Rust. Powered by [`ratatui`](https://github.com/ratatui/ratatui) and [`crossterm`](https://github.com/crossterm/crossterm).

TUI Typer drops you straight into a 30-second focus window. No bloat, no complex menus—just type. The interface tracks your progress in real-time, marks errors with subtle styling, and rewards you with a retro ASCII trophy screen displaying your calculated Words Per Minute (WPM) when the timer hits zero.

## ✨ Features

* **Minimalist Design**: Zero-friction startup. The game starts tracking your time the second you hit your first key.
* **Instant Visual Feedback**: Dynamic character highlighting using a distinct cyberpunk color palette (Lime green for hits, rosy red with underlines for misses).
* **Robust Safety Fallbacks**: Safely reads custom dictionary streams from an `english_short.txt` file, defaulting seamlessly to internal word variants if the resource is missing.
* **Retro Statistics**: Custom-rendered ASCII art summary dashboard showing a trophy and huge digital readouts for your exact WPM score.
* **Fully Unit Tested**: Deep internal coverage tracking word-matching algorithms, WPM calculation logic, and state engine transitions.

---

## 🛠️ Installation & Building

Since TUI Typer is built entirely in Rust, you can compile and build the binary directly using `cargo`.

### Prerequisites

Make sure you have the Rust toolchain installed. If not, pick it up via [rustup.rs](https://rustup.rs/).

### Quick Start

1. Clone the repository:

   ```bash
   git clone [https://github.com/your-username/tui-typer.git](https://github.com/your-username/tui-typer.git)
   cd tui-typer
