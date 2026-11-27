# 🌻 Zombie Plant — GUI

Versi **grafis** dari game terminal `zombie-plant`, dibuat dengan Rust + [`macroquad`](https://crates.io/crates/macroquad).
Logika game dipakai ulang dari versi terminal, sementara tampilannya memakai **sprite pixel art 16×16 yang didefinisikan sebagai peta piksel di kode** (`src/pixel_art.rs`) dan dirender dengan filter *nearest* supaya kotak pikselnya tetap tajam — **tanpa file gambar sprite**. Teksnya juga memakai font pixel **Silkscreen Bold** (`assets/Silkscreen-Bold.ttf`, lisensi OFL) supaya gaya retro-nya konsisten dengan sprite.

> **Catatan:** seperti proyek terminalnya, ini adalah hasil **murni *vibe coding***: dibangun lewat
> iterasi percakapan, tanpa spesifikasi formal atau desain arsitektur di awal.

## Menjalankan

```powershell
cargo run
```

Jendela tetap **1280×720**, tidak bisa di-resize, dan tombol maximize tidak tersedia. Butuh kartu grafis + OpenGL yang tersedia.
Audio memakai `rodio`; jika tidak ada perangkat audio, game tetap berjalan tanpa suara.

Build rilis:

```powershell
cargo build --release
```

## Layar menu

Saat game dibuka, muncul menu dengan empat pilihan: **Start Game**, **Level Select**, **How to Play**, dan **Quit**.

- **Start Game** selalu memulai **Easy**.
- **Level Select** menampilkan tiga difficulty: **Easy / Medium / Hard**, masing-masing endless. **Medium** terbuka setelah Easy mencapai **wave 10**; **Hard** terbuka setelah Medium mencapai **wave 15**.
- **How to Play** berisi ringkasan kontrol.
- Menu bisa dikendarai dengan Arrow/WASD + Enter, atau langsung diklik dengan mouse.
- Tekan `Esc` saat bermain untuk kembali ke menu; `Q` keluar dari game.

## Kontrol

| Aksi | Keyboard | Mouse |
| --- | --- | --- |
| Pilih tanaman | `1` Sunflower, `2` Peashooter, `3` Wall-nut | Klik kartu di atas arena |
| Pindah kursor | Arrow keys / `WASD` | — |
| Tanam di petak | `Space` / `Enter` (saat bermain) | Klik petak kosong |
| Cabut tanaman | `X` (di kursor) | Klik kanan petak berisi tanaman |
| Ambil matahari | `C` (terdekat dengan kursor) | Klik matahari `🌞` |
| Jeda / lanjut | `P` | — |
| Suara on/off | `M` | — |
| Kembali ke Easy | `N` | — |
| Ulangi difficulty | `R` (setelah kalah) | — |
| Kembali ke menu | `Esc` | — |
| Keluar dari game | `Q` (di menu: `Q` / `Esc`) | Pilih **Quit** di menu |

## Yang berbeda dari versi terminal

- Emoji diganti **sprite pixel art**: 🌻 Sunflower, 🌱 Peashooter, 🥜 Wall-nut, 🧟 Zombie, matahari, dan pea. Masing-masing 16×16 piksel dengan palet terbatas dan filter *nearest* (tanpa blur). Zombie sengaja menghadap **kiri** dengan tangan terulur, mengikuti arah geraknya dari kanan ke kiri.
- Zombie punya **animasi jalan 2 frame** (kaki bergantian + sedikit *bob*) yang hanya berjalan saat zombie benar-benar bergerak; saat menggigit tanaman, gerakannya diganti getaran kecil.
- **Partikel pixel** muncul saat menanam, menembak, kena pea, menggigit, mengumpulkan matahari, dan zombie mati. Arahnya mengikuti kejadian (misalnya pea ke kanan, percikan matahari ke atas) dan dihitung dari umur efek tanpa daftar partikel yang terus dipakai.
- **Preview tanam**: petak yang ditunjuk mouse (atau kursor keyboard) menampilkan bayangan tanaman; hijau jika bisa ditanam, merah jika terisi/sun kurang.
- **Angka melayang** `+40`, `-100`, dan `+100` muncul saat mengumpulkan matahari, menanam, dan membunuh zombie. Tanaman juga bereaksi: Sunflower sedikit membesar, Peashooter mundur saat menembak, dan Wallnut retak sesuai HP.
- **Wave endless**: setiap 4 zombie membentuk satu wave. Awal wave diberi banner `WAVE n INCOMING!` dan memunculkan dua zombie sekaligus, supaya ada ritme tenang → persiapan → serangan. Tiap wave berikutnya membuat spawn lebih cepat, zombie lebih kuat, dan zombie lebih cepat.
- **Pause**: tekan `P` kapan saja saat bermain untuk membekukan waktu, spawn, tembakan, partikel, dan aksi (tanam/cabut/ambil matahari). Musik latar ikut berhenti; overlay `PAUSED` menampilkan petunjuk `P` lanjut, `Esc` menu, `Q` keluar. `M` tetap bisa mematikan suara saat jeda.
- **Lawnmower sekali pakai per baris**: jika zombie lolos ke ujung kiri, mesin pemotong hanya menangkap zombie yang benar-benar sampai ke tepi. Zombie lain yang masih berjalan di baris itu tetap hidup, jadi lawnmower bukan jalan pintas menang; escape kedua di baris yang sama tetap kalah.
- **Sistem bintang**: 1 bintang jika run mencapai wave 10, +1 jika mencapai wave 20, +1 jika tidak memakai lawnmower sama sekali. Bintang dihitung saat run berakhir (kalah) dan ditampilkan di menu serta kotak hasil.
- Tanaman bisa dicabut dengan `X`/klik kanan dan mendapat **refund 25 sun**, jadi salah tanam tidak langsung merusak run.
- Generator gambar eksternal tidak dipakai: sprite dibuat dari peta karakter + palet warna langsung di `src/pixel_art.rs`, jadi tetap bisa diedit sebagai teks.
- File PNG asli juga disediakan di `assets/sprites/` (masing-masing 16×16) plus `preview_8x.png` untuk melihat semuanya sekaligus. PNG diekspor dari peta yang sama; regenerate dengan `python tools/export_sprites.py` setelah mengubah peta.
- Semua elemen (judul, HUD, kartu, arena, petunjuk, kotak hasil) tetap disejajarkan di tengah jendela dan otomatis menyesuaikan ukuran saat resize.
- Progress bar berupa 20 segmen kotak hijau/abu yang menunjukkan progres menuju wave berikutnya, bukan karakter `▓`/`░`.
- Teks memakai font pixel **Silkscreen Bold** yang dibundled, bukan font bitmap bawaan macroquad. Ukurannya sengaja kelipatan 8 (16/24/32 px) agar grid piksel font tetap sejajar dan tidak blur; HUD tetap muat di jendela 1280 px.
- Efek suara dan musik latar tetap disintesis dari kode.
- Grid 5×12 (sama seperti versi terminal), tapi mode GUI memakai tiga difficulty endless dengan kurva kesulitan sendiri.
- **Best wave, waktu bertahan terlama, skor tertinggi, dan bintang per difficulty** disimpan otomatis ke `%APPDATA%\ZombiePlantGUI\records.txt` (atau file `zombie-plant-gui-records.txt` di folder game jika `APPDATA` tidak ada). Format baru adalah `level wave waktu skor bintang`; file lama `level waktu skor [bintang]` tetap terbaca dan dimigrasikan ke wave unlock yang setara. Kotak hasil menampilkan label `NEW BEST WAVE!` / `NEW LONGEST RUN!` / `NEW HIGH SCORE!` / `NEW STARS!` saat rekor dipecah.
- Skor dihitung dari `kills × 100 + sisa matahari × 2 + wave × 250 + detik bertahan × 5`, jadi bertahan lebih lama selalu menaikkan skor.

## Struktur proyek

| File | Isi |
| --- | --- |
| `src/main.rs` | Window, loop 60 FPS, input keyboard/mouse, wiring audio, perpindahan menu ↔ game |
| `src/menu.rs` | Menu awal, level select (dengan unlock), help, dan hit-testing mouse |
| `src/game.rs` | State game, difficulty endless, scaling wave, shovel, lawnmower, bintang, kombat |
| `src/render.rs` | Layout adaptif, sprite + animasi, partikel, angka melayang, preview, HUD, kotak hasil |
| `src/pixel_art.rs` | Peta piksel 16×16 + palet untuk 8 frame sprite (termasuk 2 frame jalan zombie dan lawnmower) |
| `src/audio.rs` | Sintesis efek suara (termasuk shovel, pause, wave, milestone, lawnmower) + sequencer musik via `rodio` |
| `src/rng.rs` | PRNG xorshift kecil |
| `src/records.rs` | Rekor best wave, waktu bertahan, skor, dan bintang per difficulty (tersimpan di disk) |
| `assets/Silkscreen-Bold.ttf` | Font pixel TTF untuk seluruh teks di game |
| `assets/Silkscreen-OFL.txt` | Lisensi SIL Open Font License untuk font tersebut |
| `assets/sprites/*.png` | Ekspor PNG 16×16 dari peta piksel (8 frame + `preview_8x.png`) |
| `tools/export_sprites.py` | Script Python (stdlib) untuk mengekspor peta piksel ke PNG |

## Tes

```powershell
cargo test
```

Tes mencakup seluruh logika game (ekonomi, combat, endless yang tidak pernah menang,
skaling difficulty), shovel & refund, pause yang membekukan simulasi dan menolak aksi,
bintang berbasis wave, wave besar dan milestone, lawnmower sekali pakai, angka melayang,
pemetaan koordinat mouse, layout terpusat di renderer, penyimpanan rekor (termasuk
migrasi file lama), navigasi & unlock berbasis wave di menu, fase jalan zombie, durasi
partikel, arah pancaran partikel, serta
validasi bahwa setiap peta sprite berisi 16 baris dan hanya memakai warna palet yang
dikenal.
