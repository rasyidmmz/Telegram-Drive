<div align="center">

![TeleStash Hero Banner](docs/assets/telestash_hero_banner.jpg)

# TeleStash

### *Your Unlimited Personal Cinema Cloud & High-Speed Media Engine*

[![Release](https://img.shields.io/github/v/release/rasyidmmz/Telegram-Drive?style=for-the-badge&color=06B6D4&labelColor=0F172A)](https://github.com/rasyidmmz/Telegram-Drive/releases/latest)
[![Platform](https://img.shields.io/badge/Platform-Windows_11_x64-0284C7?style=for-the-badge&logo=windows&labelColor=0F172A)](https://github.com/rasyidmmz/Telegram-Drive/releases/latest)
[![Engine](https://img.shields.io/badge/Video_Engine-MPV_Native_Sidecar-8B5CF6?style=for-the-badge&labelColor=0F172A)](https://mpv.io)
[![License](https://img.shields.io/badge/License-MIT-10B981?style=for-the-badge&labelColor=0F172A)](LICENSE)

<br/>

**TeleStash** is a high-performance Windows 11 desktop app that turns Telegram's unlimited cloud storage into your private, buffer-free **Plex-style Personal Cinema Vault**.

Built with **Tauri v2, Rust, React, and native MPV sidecar**, TeleStash lets you stream 4K/1080p HEVC movies and TV series directly from the cloud with zero local disk footprint, automatic Whisper AI English subtitles, and multi-worker MTProto parallel transfer pools.

[**Download Latest Release (v1.9.34)**](https://github.com/rasyidmmz/Telegram-Drive/releases/latest) · [**Brand System**](#-brand-identity--visual-system) · [**Build Instructions**](#-build-from-source)

</div>

---

## 🎨 Brand Identity & Visual System

![TeleStash Brand Kit](docs/assets/telestash_brand_kit.jpg)

### Core Metaphor: The Vault & The Stream
TeleStash combines **The Stash** (*an unlimited, private digital vault*) with **The Stream** (*instant, cinema-quality video playback*).

* **Electric Cyan (`#06B6D4`)**: Represents high-speed data transmission & MTProto multi-worker streams.
* **Midnight Blue (`#0F172A`)**: Represents the dark-mode personal cinema viewing environment.
* **Slate Charcoal (`#1E293B`)**: Minimalist, distraction-free desktop container UI.

---

## 🚀 Key Features

### 🎬 Plex-Class Personal Media Hub
* **Direct MPV Cinema Engine**: Bundled native MPV sidecar supports 4K HEVC/H.265, MKV, MP4, and surround audio without relying on limited browser codecs.
* **Zero-Disk Streaming**: Streams video chunks 100% in-memory with a **16 MB ring buffer** and forward pre-fetching—leaving 0 Bytes of leftover video cache on your disk.
* **Instant Token Auth**: Fail-proof stream authorization with embedded query tokens for flawless playback resume.

### 🎙️ Automatic Whisper AI English Subtitles
* **1-Click & Batch Transcription**: Generates crisp `.en.srt` subtitles for movies and full TV show seasons using local Whisper AI binary.
* **100x CPU Accelerated Audio Extraction**: Leverages MPV `--benchmark` extraction, processing a 2-hour movie's audio track in under 3 seconds.
* **Auto-Upload to Cloud**: Automatically uploads generated `.en.srt` subtitle files back to your Telegram folder for future watching.
* **System-Friendly Priority**: Runs at Win32 `BELOW_NORMAL_PRIORITY_CLASS` with thread capping (max 2 threads) to prevent CPU spikes.

### ⚡ Ultra-Fast MTProto Transfer Engine
* **4-Worker Parallel Upload Pool**: Streams 512 KB chunks concurrently to Telegram Data Centers for **3x–5x faster upload speeds**.
* **4-Worker Parallel Download Pool**: Downloads disjoined MTProto chunk ranges simultaneously for instant file retrieval.
* **SQLite Resumable Upload Checkpoints**: Automatically saves upload progress per part. Interrupted uploads resume from part $N$ instead of starting over at 0%.
* **Unlimited Large-File Splitting**: Files above 2 GB are automatically split into 512 MB part messages with a `.tdmanifest.json` manifest, rendered seamlessly as a single file in TeleStash.

### 🛡️ Clean & Ad-Free Experience
* **100% Ad-Free**: Clean, modern dark-mode user interface.
* **Session Health & Keep-Alive**: 60s background MTProto ping loop repairs sessions and refreshes peer caches after system sleep or network reconnects.
* **Terminal Diagnostic Console**: Real-time monospace event log with microsecond timestamps for full transfer visibility.

---

## 💻 System Requirements

| Specification | Minimum Requirement |
| :--- | :--- |
| **Operating System** | Windows 11 (64-bit) |
| **Processor** | 64-bit Dual-Core CPU |
| **Memory** | 4 GB RAM |
| **Graphics** | DirectX 11 compatible GPU |
| **Network** | Broadband Internet Connection |

---

## ⬇️ Installation

Download the official setup installer from the [Releases](https://github.com/rasyidmmz/Telegram-Drive/releases/latest) page:

```
TeleStash_1.9.34_x64-setup.exe
```

1. Run `TeleStash_1.9.34_x64-setup.exe` and follow the prompt.
2. Sign in with your Telegram API Credentials (API ID & API Hash from [my.telegram.org](https://my.telegram.org)).
3. Enjoy your personal, unlimited cinema cloud!

---

## 🛠️ Build From Source

### Prerequisites
- Node.js (v18+)
- Rust (Stable Toolchain)
- Visual Studio Build Tools with **Desktop development with C++**
- WebView2 Runtime

### Setup & Run
```powershell
# Clone the repository
git clone https://github.com/rasyidmmz/Telegram-Drive.git
cd Telegram-Drive\app

# Install dependencies & run in dev mode
npm install
npm run tauri dev
```

### Build Production NSIS Installer
```powershell
npm run tauri build
```

---

## 📄 License & Disclaimer

TeleStash is licensed under the **MIT License**.

> **Disclaimer**: TeleStash is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Telegram FZ-LLC or Plex Inc. Please use TeleStash responsibly and in full compliance with Telegram's Terms of Service.
