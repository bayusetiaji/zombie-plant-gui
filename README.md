# 🌻 Zombie Plant — GUI

Versi **grafis** dari game terminal `zombie-plant`, dibuat dengan Rust + [`macroquad`](https://crates.io/crates/macroquad).
Logika game dipakai ulang dari versi terminal, sementara tampilannya memakai **sprite pixel art 16×16 yang didefinisikan sebagai peta piksel di kode** (`src/pixel_art.rs`) dan dirender dengan filter *nearest* supaya kotak pikselnya tetap tajam — **tanpa file gambar sprite**. Teksnya juga memakai font pixel **Silkscreen Bold** (`assets/Silkscreen-Bold.ttf`, lisensi OFL) supaya gaya retro-nya konsisten dengan sprite.

> **Catatan:** seperti proyek terminalnya, ini adalah hasil **murni *vibe coding***: dibangun lewat
> iterasi percakapan, tanpa spesifikasi formal atau desain arsitektur di awal.

## Menjalankan

```powershell
cargo run
```

Jendela awal **1280×720** dan bisa di-resize. Butuh kartu grafis + OpenGL yang tersedia.
Audio memakai `rodio`; jika tidak ada perangkat audio, game tetap berjalan tanpa suara.

Build rilis:

```powershell
cargo build --release
```

## Layar menu

Saat game dibuka, muncul menu dengan empat pilihan: **Start Game**, **Level Select**, **How to Play**, dan **Quit**.

- **Start Game** selalu memulai level 1.
- **Level Select** menampilkan 3 level beserta rekor masing-masing. Level 2 terbuka setelah level 1 pernah diselesaikan, level 3 terbuka setelah level 2 — jadi progresi tetap terasa.
- **How to Play** berisi ringkasan kontrol.
- Menu bisa dikendarai dengan Arrow/WASD + Enter, atau langsung diklik dengan mouse.
- Tekan `Esc` saat bermain untuk kembali ke menu; `Q` keluar dari game.

## Kontrol

| Aksi | Keyboard | Mouse |
| --- | --- | --- |
| Pilih tanaman | `1` Sunflower, `2` Peashooter, `3` Wall-nut | Klik kartu di atas arena |
| Pindah kursor | Arrow keys / `WASD` | — |
| Tanam di petak | `Space` / `Enter` (saat bermain) | Klik petak kosong |
| Ambil matahari | `C` (terdekat dengan kursor) | Klik matahari `🌞` |
| Lanjut level | `Enter` (setelah menang) | — |
| Suara on/off | `M` | — |
| Kembali ke level 1 | `N` | — |
| Ulangi level | `R` (setelah menang/kalah) | — |
| Kembali ke menu | `Esc` | — |
| Keluar dari game | `Q` (di menu: `Q` / `Esc`) | Pilih **Quit** di menu |

## Yang berbeda dari versi terminal

- Emoji diganti **sprite pixel art**: 🌻 Sunflower, 🌱 Peashooter, 🥜 Wall-nut, 🧟 Zombie, matahari, dan pea. Masing-masing 16×16 piksel dengan palet terbatas dan filter *nearest* (tanpa blur). Zombie sengaja menghadap **kiri** dengan tangan terulur, mengikuti arah geraknya dari kanan ke kiri.
- Zombie punya **animasi jalan 2 frame** (kaki bergantian + sedikit *bob*) yang hanya berjalan saat zombie benar-benar bergerak; saat menggigit tanaman, gerakannya diganti getaran kecil.
- **Partikel pixel** muncul saat menanam, menembak, kena pea, menggigit, mengumpulkan matahari, dan zombie mati. Arahnya mengikuti kejadian (misalnya pea ke kanan, percikan matahari ke atas) dan dihitung dari umur efek tanpa daftar partikel yang terus dipakai.
- Generator gambar eksternal tidak dipakai: sprite dibuat dari peta karakter + palet warna langsung di `src/pixel_art.rs`, jadi tetap bisa diedit sebagai teks.
- File PNG asli juga disediakan di `assets/sprites/` (masing-masing 16×16) plus `preview_8x.png` untuk melihat semuanya sekaligus. PNG diekspor dari peta yang sama; regenerate dengan `python tools/export_sprites.py` setelah mengubah peta.
- Semua elemen (judul, HUD, kartu, arena, petunjuk, kotak hasil) tetap disejajarkan di tengah jendela dan otomatis menyesuaikan ukuran saat resize.
- Progress bar wave berupa 20 segmen kotak hijau/abu, bukan karakter `▓`/`░`.
- Teks memakai font pixel **Silkscreen Bold** yang dibundled, bukan font bitmap bawaan macroquad. Ukurannya sengaja kelipatan 8 (16/24/32 px) agar grid piksel font tetap sejajar dan tidak blur; HUD tetap muat di jendela 1280 px.
- Efek suara dan musik latar tetap disintesis dari kode.
- Grid 5×12 dan 3 level dengan konfigurasi yang sama dengan versi terminal.
- **High score & waktu terbaik per level** disimpan otomatis ke `%APPDATA%\ZombiePlantGUI\records.txt` (atau file `zombie-plant-gui-records.txt` di folder game jika `APPDATA` tidak ada). HUD menampilkan rekor level saat ini; kotak hasil menampilkan skor, waktu, dan label `NEW RECORD!` / `NEW BEST TIME!` / `NEW HIGH SCORE!` saat rekor dipecah.
- Skor dihitung dari `kills × 100 + sisa matahari × 2 + bonus waktu` (bonus maksimal untuk clear di bawah 180 detik).

## Struktur proyek

| File | Isi |
| --- | --- |
| `src/main.rs` | Window, loop 60 FPS, input keyboard/mouse, wiring audio, perpindahan menu ↔ game |
| `src/menu.rs` | Menu awal, level select (dengan unlock), help, dan hit-testing mouse |
| `src/game.rs` | State game, level, spawning, kombat (dipakai ulang) |
| `src/render.rs` | Layout adaptif, sprite + animasi jalan, partikel, HUD, kotak hasil, pemetaan mouse |
| `src/pixel_art.rs` | Peta piksel 16×16 + palet untuk 7 frame sprite (termasuk 2 frame jalan zombie), diubah jadi tekstur *nearest* |
| `src/audio.rs` | Sintesis efek suara + sequencer musik via `rodio` |
| `src/rng.rs` | PRNG xorshift kecil |
| `src/records.rs` | Rekor waktu tercepat & skor tertinggi per level (tersimpan di disk) |
| `assets/Silkscreen-Bold.ttf` | Font pixel TTF untuk seluruh teks di game |
| `assets/Silkscreen-OFL.txt` | Lisensi SIL Open Font License untuk font tersebut |
| `assets/sprites/*.png` | Ekspor PNG 16×16 dari peta piksel (7 frame + `preview_8x.png`) |
| `tools/export_sprites.py` | Script Python (stdlib) untuk mengekspor peta piksel ke PNG |

## Tes

```powershell
cargo test
```

Tes mencakup seluruh logika game (ekonomi, combat, kalah/menang, progresi level),
pemetaan koordinat mouse, layout terpusat di renderer, penyimpanan rekor,
navigasi & unlock level di menu, fase jalan zombie, durasi partikel, arah pancaran
partikel, serta validasi bahwa setiap peta sprite berisi 16 baris dan hanya
memakai warna palet yang dikenal.
