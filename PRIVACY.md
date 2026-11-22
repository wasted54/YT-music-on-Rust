# Privacy

Wasted has no accounts of its own and collects nothing about you. It only talks to the services below, and only for the features that need them:

- **YouTube Music and Google** (`music.youtube.com`, `youtube.com`, `youtubei.googleapis.com`, `googlevideo.com`, `googleusercontent.com`, `accounts.google.com`) for browsing, playback, likes, playlists, uploads and sign-in. Your sign-in cookie stays on your PC, in Credential Manager, Keychain or your keyring.
- **Better Lyrics** (`lyrics.api.dacubeking.com`, `unison.betterlyrics.org`) receives the song's YouTube ID, title, artist, album and length to find synced lyrics, and the lyric lines when you turn on translation or romanization.
- **Google Translate** (`translate.googleapis.com`, `clients5.google.com`) receives lyric lines that Better Lyrics couldn't translate or romanize.
- **SponsorBlock** (`sponsor.ajay.app`) receives the first four characters of a hash of the video's ID, to find non-music parts of music videos to skip. It never sees which video is playing.
- **Last.fm** (`ws.audioscrobbler.com`), only after you connect your Last.fm account, receives the songs you play (title, artist, album and when you played them).
- **ListenBrainz** (`api.listenbrainz.org`), only after you connect your ListenBrainz account, receives the songs you play (title, artist, album, length, YouTube Music link and when you played them).
- **Spotify** (`spotify.com`), only when you use **Transfer from Spotify**, to read the playlists you pick. Your Spotify sign-in stays on your PC. The transfer also downloads a public file from `code.thetadev.de` that it needs to read from Spotify.
- **GitHub** (`github.com`, `api.github.com`) to check for and download app updates (you can turn this off in **Settings › About › Updates**), and to download `yt-dlp` when an age-restricted song is played on an account without Premium.
- **Discord**, through the Discord app on your PC, when **Show what I'm playing on Discord** is on.
- **AirPlay speakers on your local network**, only when **Settings › Network › Allow local network scan** is on: the app looks for them (mDNS), and the one you pick as the output device receives the music you play, the app's name and your PC's network address. Nothing leaves your local network.

The app itself sends no analytics, telemetry or crash reports.
