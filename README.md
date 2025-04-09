# mpvlayer

this is a **work in progress**

a tui wrapper for mpv

## requirements
### stuff you probably have to install
yt-dlp
mpv

### stuff you probably dont have to install
kill
pidof

## quickstart
#### install yt-dlp:
```
curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o ~/.local/bin/yt-dlp
chmod a+rx ~/.local/bin/yt-dlp
```
or
```
python3 -m pip3 install --user yt-dlp
```
or with your favorite package manager

(https://github.com/yt-dlp/yt-dlp/wiki/Installation)

#### install mpv:
use your favorite package manager
or
https://mpv.io/installation/

## screenshots
![ui](assets/ui.png)

## progress
- [x] ui
- [x] play/pause
- [x] queue
- [x] repeat
- [x] shuffle
- [ ] add playlists from youtube
- [ ] song caching
- [ ] edit playlists
- [ ] 