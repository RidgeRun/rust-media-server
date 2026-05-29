# Rust media server

A Rust media server using the Rust GstC API from RidgeRun's GstD.

The official Wiki for the Media Server is in the following link: [Rust Media Server Wiki](https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon_-_Rust_Media_Server_Using_GstD_Rust_API_and_GStreamer)

## Prerequisites

- GStreamer. Version >=1.24 in order for the pipelines in this repo to work
without modifications of its elements.

### Install Commands
```sh
sudo apt update
sudo apt install -y pkg-config curl build-essential
sudo apt install -y \
  libgstreamer1.0-dev \
  gstreamer1.0-tools \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- Gstd. Install with the instructions in the following link: [GstD](https://developer.ridgerun.com/wiki/index.php/GStreamer_Daemon_-_Building_GStreamer_Daemon).
- Interpipes. Install with the instructions in the following link: [GstInterpipe](https://developer.ridgerun.com/wiki/index.php/GstInterpipe_-_Building_and_Installation_Guide).


## Layout

- `src/bin/media_server.rs`: executable entry point for the media server.

## Build

Build the executable with Cargo:

```sh
cargo build --bin media_server
```

Build a release binary:

```sh
cargo build --release --bin media_server
```

## Pre-commit Hooks

Install the repository hooks with:

```sh
pre-commit install
```

Configured checks:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`

## Run

### CLI Arguments

The media server accepts runtime parameters from the command line:

- `-c, --config <PATH>`: main YAML config file.
- `-l, --log-file <PATH>`: log output file.
- `-v, --log-level <0..4>`:
  - `0` = `KNone`
  - `1` = `KError`
  - `2` = `KWarning`
  - `3` = `KInfo`
  - `4` = `KDebug`
- `--log-append <true|false>`:
  - `true` appends to the existing log file
  - `false` truncates/overwrites the log file on startup

Default values are the following:

- `cfg/media_server.yaml`
- `media_server.log`
- `4` (`KDebug`)
- `true` (append mode)

Run with default values:

```sh
cargo run --bin media_server
```

Optional: run with GStreamer debug output:

```sh
GST_DEBUG=2 cargo run --bin media_server
```

Run with custom values:

```sh
cargo run --bin media_server -- \
  --config cfg/media_server.yaml \
  --log-file /tmp/media_server.log \
  --log-level 3 \
  --log-append false
```

### What the media server does

- It reads `cfg/media_server.yaml`, then loads each source/feature pipeline config referenced there.
- It creates and plays enabled pipelines, and writes app logs to `media_server.log`.
- With log level set to debug in `src/bin/media_server.rs`, pipeline create/play descriptions are written to the log.
- The server creates output directories on startup: `recordings/` and `snapshots/` where
the corresponding files of the recordings and snapshot features will be saved if those features are enabled.
- The media server has a streaming feature, to visualize the streams
you can run the following pipelines:
For source_cam0:
```sh
gst-launch-1.0 -e udpsrc port=5005 ! application/x-rtp,media=video,payload=96,clock-rate=90000,encoding-name=H264 ! rtpjitterbuffer latency=0 ! rtph264depay ! h264parse ! avdec_h264 ! queue max-size-buffers=1 leaky=downstream ! videoconvert ! autovideosink sync=false
```
For source_cam1:
```sh
gst-launch-1.0 -e udpsrc port=5006 ! application/x-rtp,media=video,payload=96,clock-rate=90000,encoding-name=H264 ! rtpjitterbuffer latency=0 ! rtph264depay ! h264parse ! avdec_h264 ! queue max-size-buffers=1 leaky=downstream ! videoconvert ! autovideosink sync=false
```

### Stop the server

Stop the server with `Ctrl+C`.

## Configure Pipelines

Main config file:

- `cfg/media_server.yaml`
  - Lists source config files under `sources:`.
  - Maps feature names to feature template files under `features:`.

Per-source config files, each file corresponds to the config of an individual
source of the media server, such as a camera or a video stream:

- `cfg/sources/source_cam0.yaml`
- `cfg/sources/source_cam1.yaml`

Each source file controls:

- Source enable/disable: `source.enabled: true|false`
- Source pipeline graph: `source.source_pipeline.description`
- Feature enable/disable:
  - `features.recording.enabled`
  - `features.snapshot.enabled`
  - `features.streaming.enabled`
- Feature overrides such as `filename` and `port`

Feature template files, these are pipelines with specific features for the 
media server, they each connect to the sources with Interpipes. These pipelines
can be attached to any source, the media server will create a pipeline for each one:

- `cfg/features/recording.yaml`: Recording feature pipeline
- `cfg/features/snapshot.yaml`: Snapshot feature pipeline
- `cfg/features/streaming.yaml`: Streaming feature pipeline

Templates define reusable pipeline snippets with placeholders (for example `${interpipe_sink}` and `${filename}`) that are filled using each source file override.

## Enable or Disable Pipelines

Disable an entire source:

```yaml
source:
  enabled: false
```

Disable one feature for a source:

```yaml
features:
  streaming:
    enabled: false
```

## Add an override to a feature pipeline:

See the following patch as an example of how to add an override to the feature
pipelines:
```sh
diff --git a/cfg/features/recording.yaml b/cfg/features/recording.yaml
index af9725a..845dd54 100644
--- a/cfg/features/recording.yaml
+++ b/cfg/features/recording.yaml
@@ -6,4 +6,4 @@ recording:
     queue max-size-buffers=1 leaky=downstream !
     x264enc tune=zerolatency bitrate=2000 speed-preset=veryfast !
     h264parse config-interval=1 !
-    splitmuxsink location=${filename} max-size-time=5000000000 muxer-factory=mp4mux
\ No newline at end of file
+    splitmuxsink location=${filename} max-size-time=${duration} muxer-factory=mp4mux
diff --git a/cfg/sources/source_cam0.yaml b/cfg/sources/source_cam0.yaml
index ed173b9..de2a59e 100644
--- a/cfg/sources/source_cam0.yaml
+++ b/cfg/sources/source_cam0.yaml
@@ -17,10 +17,11 @@ source:

 features:
   recording:
-    enabled: false
+    enabled: true
     pipeline_id: "cam0_recording"
     overrides:
       filename: "recordings/cam0_recording_%05d.mp4"
+      duration: "5000000000"

   snapshot:
     enabled: false
```

Note: You can change the descriptions of any source and feature pipeline
as long as the overall structure of the overrides and the rest of the parameters
remain the same. 

After changing YAML, restart the media server to apply the new pipeline set.
