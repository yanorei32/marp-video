# Marp Video

## Example Config

```toml
#  same dir: ".marp-video"
#  home dir: "$HOME/.marp-video"
# cache dir: "$XDG_CACHE_HOME/marp-video"
#   tmp dir: "/tmp/.marp-video"
cache_dir = ".marp-video"

tts.default.envs.SPEED = "1.1"

dep.ffprobe = {}

[dep.ffmpeg]
global_args = [
  "-pix_fmt", "yuv420p",
  "-acodec", "aac",
  "-vcodec", "libx264",
  "-r", "30",
]

[dep.marp]
global_args = [
  "--browser", "firefox",
  "--allow-local-files",
]

[profile.default]
ffmpeg_args = ["-crf", "30"]

[profile.fast]
ffmpeg_args = ["-b:v", "2000k", "-preset", "veryfast"]
marp_args = ["--image-scale", "0.5"]

[profile.4k]
ffmpeg_args = ["-crf", "20"]
marp_args = ["--image-scale", "3"]
width = 3840
height = 2160
```
