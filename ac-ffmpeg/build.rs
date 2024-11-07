use std::path::Path;

use cc::Build;

fn main() {
    let docs_rs = std::env::var_os("DOCS_RS");

    let ffmpeg_features = ac_ffmpeg_features::ffmpeg_features(docs_rs.is_some());

    for feature in &ffmpeg_features {
        println!("cargo:rustc-cfg={}", feature);
    }

    // skip building native resources during docs.rs builds
    if docs_rs.is_some() {
        return;
    }

    let src_dir = Path::new("src");

    let src_format_dir = src_dir.join("format");
    let src_codec_dir = src_dir.join("codec");
    let src_codec_audio_dir = src_codec_dir.join("audio");
    let src_codec_video_dir = src_codec_dir.join("video");

    println!("cargo:rerun-if-changed={}", src_dir.display());

    let mut build = Build::new();

    build.include(src_dir);

    for dir in ac_ffmpeg_build::ffmpeg_include_dirs(true) {
        build.include(dir);
    }

    for feature in &ffmpeg_features {
        build.define(&format!("FFW_FEATURE_{}", feature.to_uppercase()), None);
    }

    build
        .file(src_dir.join("error.c"))
        .file(src_dir.join("logger.c"))
        .file(src_dir.join("packet.c"))
        .file(src_dir.join("time.c"))
        .file(src_format_dir.join("demuxer.c"))
        .file(src_format_dir.join("io.c"))
        .file(src_format_dir.join("muxer.c"))
        .file(src_format_dir.join("stream.c"))
        .file(src_codec_dir.join("bsf.c"))
        .file(src_codec_dir.join("mod.c"))
        .file(src_codec_dir.join("frame.c"))
        .file(src_codec_audio_dir.join("resampler.c"))
        .file(src_codec_video_dir.join("scaler.c"))
        .compile("ffwrapper");

    // build.include(Path::new("/opt/homebrew/opt/zlib/include"));

    for dir in ac_ffmpeg_build::ffmpeg_lib_dirs(true) {
        println!("cargo:rustc-link-search=native={}", dir.display());
    }

    let ffmpeg_link_mode = link_mode();

    // Added this to fix linking errors on Windows
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/zlib/lib/");
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/x264/lib/");
        println!("cargo:rustc-link-search=native=/opt/homebrew/opt/bzip2/lib/");

        println!("cargo:rustc-link-lib=framework=CoreServices");
        println!("cargo:rustc-link-lib=framework=CoreGraphics");
        println!("cargo:rustc-link-lib=framework=CoreText");
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=AudioUnit");
        println!("cargo:rustc-link-lib=framework=AudioToolbox");
        println!("cargo:rustc-link-lib=framework=CoreAudio");
        println!("cargo:rustc-link-lib=framework=Security");
        println!("cargo:rustc-link-lib=framework=VideoToolbox");

        link("z", ffmpeg_link_mode);
        link("x264", ffmpeg_link_mode);
        link("bz2", ffmpeg_link_mode);
    }

    link("avcodec", ffmpeg_link_mode);
    link("avformat", ffmpeg_link_mode);
    link("avutil", ffmpeg_link_mode);
    link("swresample", ffmpeg_link_mode);
    link("swscale", ffmpeg_link_mode);
}

/// Get the FFmpeg link mode.
fn link_mode() -> &'static str {
    std::env::var("FFMPEG_STATIC")
        .map(|v| match v.as_str() {
            "0" => "dylib",
            _ => "static",
        })
        .unwrap_or("dylib")
}

/// Link a given library.
fn link(lib: &str, mode: &str) {
    println!("cargo:rustc-link-lib={}={}", mode, lib);
}
