# Implementasi Generate English CC Lokal untuk TeleStash

## Ringkasan

Tambahkan tindakan **Generate English CC** pada menu file untuk semua format video yang dikenali TeleStash. mpv mengekstrak audio dari stream lokal menjadi WAV 16 kHz mono, lalu `whisper.cpp` CPU dengan model `base.en` membuat SRT yang otomatis dimuat ketika video dibuka kembali.

Fitur berjalan lokal, mendukung file Telegram biasa maupun split-upload, tidak mengubah video asli, dan tidak memerlukan Python, FFmpeg, API key, atau layanan cloud.

## Perubahan Utama

- Buat `EnglishCcManager` backend dengan satu pekerjaan aktif pada satu waktu:
  - Fase: `idle`, `extracting`, `transcribing`, `ready`, `error`, `cancelled`.
  - SRT disimpan sebagai `streaming/captions/{folder_id}_{message_id}.en.srt`.
  - WAV dan output parsial ditempatkan di cache sementara dan selalu dibersihkan.
  - Regenerasi menulis SRT baru secara atomik; kegagalan atau pembatalan tidak menghapus SRT lama.
- Ekstrak audio menggunakan mpv yang sudah dibundel:
  - Stream token dikirim melalui header, tidak ditaruh pada URL proses.
  - Gunakan output WAVE, mono, signed 16-bit, 16 kHz, tanpa video atau konfigurasi mpv pengguna.
  - Pendekatan ini telah diverifikasi pada binary repo: audio 60 detik diekstrak sekitar 2,35 detik. mpv resmi mendukung output PCM/WAVE dan konversi sample rate/channel. [Dokumentasi mpv](https://mpv.io/manual/master/#audio-output-drivers)
- Jalankan `whisper-cli` dengan `base.en`, English-only, CPU-only, maksimum empat thread, SRT output, pemisahan pada batas kata, dan panjang segmen maksimum 42 karakter. `whisper.cpp` mendukung Windows serta CPU-only inference. [Dokumentasi whisper.cpp](https://github.com/ggml-org/whisper.cpp)
- Ubah playback mpv agar menerima identitas folder/message dan menambahkan `--sub-file=<cached-srt>` hanya ketika SRT valid tersedia. Satu subtitle eksternal yang diberikan melalui `--sub-file` ditampilkan secara default oleh mpv. [Opsi subtitle mpv](https://mpv.io/manual/master/#options-sub-file)
- Menu file menampilkan:
  - **Generate English CC** jika belum ada.
  - **Generating English CC…** atau **Cancel English CC** ketika aktif.
  - **Regenerate English CC** jika cache tersedia.
  - Toast persisten menampilkan fase/progress; status dipoll sekitar setiap 750 ms.
- Pembersihan cache global dan per-file turut menghapus SRT terkait; model Whisper tidak ikut dihapus karena merupakan resource aplikasi.

## API dan Packaging

- Tambahkan kontrak Tauri:
  - `cmd_generate_english_cc(message_id, folder_id, force) -> EnglishCcStatus`
  - `cmd_get_english_cc_status(message_id, folder_id) -> EnglishCcStatus`
  - `cmd_cancel_english_cc(message_id, folder_id) -> ()`, bersifat idempotent.
  - `cmd_play_in_mpv(url, message_id, folder_id)` menggantikan bentuk `url`-only.
- `EnglishCcStatus` berisi `file_key`, `phase`, `progress: number | null`, `cached`, dan `error`.
- Jika video kedua diminta ketika pekerjaan lain aktif, backend menolak dengan pesan bahwa hanya satu generasi CC dapat berjalan.
- Bundel resource resmi `whisper.cpp v1.9.1`:
  - `whisper-bin-x64.zip`, SHA-256 `7d8be46ecd31828e1eb7a2ecdd0d6b314feafd82163038ab6092594b0a063539`.
  - Salin `whisper-cli.exe` dan DLL runtime yang menyertainya ke satu direktori resource.
  - Bundel `ggml-base.en.bin`, ukuran `147964211` byte, SHA-256 `a03779c86df3323075f5e796cb2ce5029f00ec8869eee3fdfb897afe36c6d002`.
  - Workflow rilis dan script setup lokal wajib gagal jika checksum atau file runtime tidak sesuai.
- Gunakan resource bundling Tauri agar executable, DLL, dan model tetap berdekatan serta dapat di-resolve dari Rust. [Dokumentasi resource Tauri](https://v2.tauri.app/develop/resources/)
- Tambahkan key UI yang sama ke seluruh katalog bahasa agar pemeriksaan i18n tetap lulus.

## Test Plan

- Unit test Rust untuk file key, path cache, argumen ekstraksi mpv, argumen Whisper, pembatasan thread, parsing progress, pemuatan `--sub-file`, dan penggantian SRT atomik.
- Uji status untuk cache siap, regenerasi, pekerjaan ganda, pembatalan saat ekstraksi/transkripsi, kegagalan proses, dan pembersihan artefak parsial.
- Verifikasi build dengan `cargo test`, `npm run build`, dan `node check-i18n.cjs`.
- Uji integrasi menggunakan MP4 biasa, split-upload MP4, MKV/WebM/MOV, video tanpa audio, nama file Unicode, dan video yang sudah memiliki subtitle embedded.
- Uji installer NSIS pada Windows bersih tanpa Python, FFmpeg, atau Whisper: generate CC harus tetap bekerja, SRT otomatis tampil di mpv, dan fitur tetap dapat digunakan offline setelah video tersedia dari Telegram.

## Asumsi

- Audio sumber berbahasa Inggris; tidak ada terjemahan, diarization, editor subtitle, embedded MP4 subtitle, atau burn-in.
- Generasi hanya dimulai secara manual dari menu file; tidak otomatis saat upload atau playback.
- Model tetap `base.en`, CPU-only, maksimum empat thread, dan satu pekerjaan bersamaan untuk menjaga laptop i5-6200U/8 GB tetap responsif.
- Playback normal tetap seperti sekarang; jika CC belum dibuat, video dibuka tanpa subtitle hasil Whisper.
