//! Platform presets for mobile, desktop, and console-oriented targets

#[derive(Debug, Clone)]
pub struct PlatformSpec {
    pub name: &'static str,
    pub triple: &'static str,
    /// Extra clang args after IR (sysroot placeholders documented)
    pub clang_args: &'static [&'static str],
    pub notes: &'static str,
}

pub fn resolve_platform(name: &str) -> Option<PlatformSpec> {
    let n = name.to_ascii_lowercase();
    Some(match n.as_str() {
        "linux" | "linux-x64" => PlatformSpec {
            name: "linux",
            triple: "x86_64-unknown-linux-gnu",
            clang_args: &["-lm"],
            notes: "Desktop Linux x86_64",
        },
        "linux-arm64" | "linux-aarch64" => PlatformSpec {
            name: "linux-arm64",
            triple: "aarch64-unknown-linux-gnu",
            clang_args: &["-lm"],
            notes: "Desktop/server Linux aarch64",
        },
        "macos" | "macos-arm64" | "darwin" => PlatformSpec {
            name: "macos",
            triple: "arm64-apple-darwin",
            clang_args: &["-lm"],
            notes: "Apple Silicon macOS",
        },
        "macos-x64" | "macos-intel" => PlatformSpec {
            name: "macos-x64",
            triple: "x86_64-apple-darwin",
            clang_args: &["-lm"],
            notes: "Intel macOS",
        },
        "windows" | "windows-x64" => PlatformSpec {
            name: "windows",
            triple: "x86_64-pc-windows-msvc",
            clang_args: &[],
            notes: "Windows x64 (MSVC ABI)",
        },
        "android" | "android-arm64" => PlatformSpec {
            name: "android",
            triple: "aarch64-linux-android",
            clang_args: &["-shared", "-fPIC"],
            notes: "Android arm64 — set ANDROID_NDK and pass --sysroot from NDK llvm prebuilt",
        },
        "android-x64" => PlatformSpec {
            name: "android-x64",
            triple: "x86_64-linux-android",
            clang_args: &["-shared", "-fPIC"],
            notes: "Android x86_64 emulator",
        },
        "ios" | "ios-arm64" => PlatformSpec {
            name: "ios",
            triple: "arm64-apple-ios",
            clang_args: &[],
            notes: "iOS arm64 — requires macOS + Xcode; use xcrun -sdk iphoneos",
        },
        "ios-sim" => PlatformSpec {
            name: "ios-sim",
            triple: "arm64-apple-ios-simulator",
            clang_args: &[],
            notes: "iOS Simulator (Apple Silicon)",
        },
        "wasm" | "wasi" => PlatformSpec {
            name: "wasm",
            triple: "wasm32-wasi",
            clang_args: &[],
            notes: "Prefer purec --emit-wasm for WASI text modules",
        },
        "console" | "console-generic" => PlatformSpec {
            name: "console",
            triple: "x86_64-unknown-linux-gnu",
            clang_args: &["-lm", "-fno-exceptions", "-ffreestanding"],
            notes: "Generic console-oriented flags; replace triple/sysroot with vendor SDK",
        },
        _ => return None,
    })
}

pub fn list_platforms() -> &'static str {
    "linux, linux-arm64, macos, macos-x64, windows, android, android-x64, ios, ios-sim, wasm, console"
}
