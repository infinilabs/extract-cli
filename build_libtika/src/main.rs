use const_format::concatcp;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const GRAALVM_JDK_PATH: &str = "./graalvm_jdk";

#[cfg(target_os = "macos")]
const LIBTIKA: &str = "libtika_native.dylib";
#[cfg(target_os = "linux")]
const LIBTIKA: &str = "libtika_native.so";
#[cfg(target_os = "windows")]
const LIBTIKA: &str = "libtika_native.dll";

const LIBTIKA_PATH_UNDER_GRADLEW: &str =
    concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", LIBTIKA);

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        const LIBJAVA: &str = "libjava.so";
        const LIBJAVA_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", LIBJAVA);

        const LIBJVM: &str = "libjvm.so";
        const LIBJVM_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", LIBJVM);

        const LIBAWT: &str = "libawt.so";
        const LIBAWT_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", LIBAWT);

        const LIBAWT_HEADLESS: &str = "libawt_headless.so";
        const LIBAWT_HEADLESS_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", LIBAWT_HEADLESS);
    } else if #[cfg(target_os = "windows")] {
        const LIBTIKA_LIB: &str = "libtika_native.lib";
        const LIBTIKA_LIB_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", LIBTIKA_LIB);

        const AWT: &str = "awt.dll";
        const AWT_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", AWT);

        const JAVA: &str = "java.dll";
        const JAVA_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", JAVA);

        const JVM: &str = "jvm.dll";
        const JVM_PATH_UNDER_GRADLEW: &str =
            concatcp!(TIKA_NATIVE, "/build/native/nativeCompile/", JVM);
    }
}

const TIKA_NATIVE: &str = "./tika-native";

fn main() {
    /*
     * Install GraalVM if not found
     */
    let graalvm_home = install_graalvm_ce(&GRAALVM_JDK_PATH.into());
    assert!(graalvm_home.exists());

    /*
     * Build the native shared library
     */
    let gradlew_filename = if cfg!(target_os = "windows") {
        "gradlew.bat"
    } else {
        "gradlew"
    };

    // We need to canonicalize these 2 paths before passing them to the Gradlew
    // script, God knows why.
    let gradlew_bin = dunce::canonicalize(Path::new(TIKA_NATIVE).join(gradlew_filename)).unwrap();
    let graalvm_home = dunce::canonicalize(graalvm_home).unwrap();
    let tika_native_canonicalized = dunce::canonicalize(Path::new(TIKA_NATIVE)).unwrap();
    assert!(gradlew_bin.exists());
    assert!(graalvm_home.exists());
    assert!(tika_native_canonicalized.exists());

    println!("Progress: building libtika");
    let status = Command::new(gradlew_bin)
        .current_dir(tika_native_canonicalized)
        .arg("--no-daemon")
        .arg("nativeCompile")
        .env("JAVA_HOME", graalvm_home)
        .status()
        .unwrap_or_else(|e| panic!("Failed to spawn child process [gradlew]: {:?}", e));
    if status.success() {
        assert!(Path::new(LIBTIKA_PATH_UNDER_GRADLEW).exists());
        println!("Progress: libtika built successfully");
    } else {
        println!(
            "Progress: failed to build libtika, gradlew status [{:?}] check the error logs above. Aborting",
            status
        );
        std::process::exit(1);
    }

    /*
     * Move the shared-library(s)
     *
     * On macOS: we only need 1 libtika_native.dylib
     * On Linux: we need:
     *     1. libtika_native.so
     *     2. libjava.so
     *     3. libjvm.so
     *     4. libawt.so
     *     5. libawt_headless.so
     */
    println!("Progress: moving shared libraries to project root");
    std::fs::copy(LIBTIKA_PATH_UNDER_GRADLEW, LIBTIKA).unwrap();
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            std::fs::copy(LIBJAVA_PATH_UNDER_GRADLEW, LIBJAVA).unwrap();
            std::fs::copy(LIBJVM_PATH_UNDER_GRADLEW, LIBJVM).unwrap();
            std::fs::copy(LIBAWT_PATH_UNDER_GRADLEW, LIBAWT).unwrap();
            std::fs::copy(LIBAWT_HEADLESS_PATH_UNDER_GRADLEW, LIBAWT_HEADLESS).unwrap();
        } else if #[cfg(target_os = "windows")] {
            std::fs::copy(JAVA_PATH_UNDER_GRADLEW, JAVA).unwrap();
            std::fs::copy(JVM_PATH_UNDER_GRADLEW, JVM).unwrap();
            std::fs::copy(AWT_PATH_UNDER_GRADLEW, AWT).unwrap();

            // Linker needs this on MSVC
            std::fs::copy(LIBTIKA_LIB_PATH_UNDER_GRADLEW, LIBTIKA_LIB).unwrap();
        }
    }
    println!("Progress: libraries moved");

    /*
     * Set Install Name on macOS
     */
    set_libtika_install_name_macos();

    println!("Progress: successfully built and moved libtika");
}

fn set_libtika_install_name_macos() {
    if cfg!(target_os = "macos") {
        println!("Progress: updating libtika Install Name");
        let status = Command::new("install_name_tool")
            .arg("-id")
            .arg(format!("@rpath/{}", LIBTIKA))
            .arg(LIBTIKA)
            .status()
            .expect("Failed to run install_name_tool on the dylib");
        assert!(status.success(), "install_name_tool -id failed");
        println!("Progress: libtika Install Name updated");
    }
}

// checks if GraalVM JDK is valid by checking if native-image is found in [graalvm_home]/bin
pub fn check_graalvm(graalvm_home: &Path, panic: bool) -> bool {
    let native_image_exe = if cfg!(target_os = "windows") {
        "native-image.cmd"
    } else {
        "native-image"
    };

    // Check that native-image is in [graalvm_home]/bin
    let native_image = graalvm_home.join("bin").join(native_image_exe);
    let exists = native_image.exists();
    if panic && !exists {
        panic!(
            "Your GraalVM JDK installation is pointing to: {}. Please make sure \
                it is a valid GraalVM JDK. {}",
            graalvm_home.display(),
            graalvm_install_help_msg()
        );
    }
    exists
}

fn graalvm_install_help_msg() -> String {
    let sdkman_graalvm_version = if cfg!(target_os = "macos") {
        "24.1.1.r23-nik" // Bellsoft Liberika r23 means jdk 23
    } else {
        "23.0.1-graalce"
    };

    format!(
        "\nWe recommend using sdkman to install and \
                manage different JDKs. See https://sdkman.io/usage for more information.\n\
                You can install graalvm using:\n  \
                sdk install java {} \n  \
                sdk use java {}",
        sdkman_graalvm_version, sdkman_graalvm_version
    )
}

pub fn install_graalvm_ce(install_dir: &PathBuf) -> PathBuf {
    println!("Progress: downloading GraalVM JDK");
    let (base_url, archive_ext, main_dir) = if cfg!(target_os = "windows") {
        let url = if cfg!(target_arch = "x86_64") {
            "https://github.com/graalvm/graalvm-ce-builds/releases/download/jdk-23.0.1/graalvm-community-jdk-23.0.1_windows-x64_bin.zip"
        } else {
            panic!("Unsupported windows architecture");
        };
        (url, "zip", "graalvm-community-openjdk-23.0.1+11.1")
    } else if cfg!(target_os = "macos") {
        let (url, dir) = if cfg!(target_arch = "x86_64") {
            (
                "https://github.com/bell-sw/LibericaNIK/releases/download/24.1.1+1-23.0.1+13/bellsoft-liberica-vm-full-openjdk23.0.1+13-24.1.1+1-macos-amd64.tar.gz",
                "bellsoft-liberica-vm-full-openjdk23-24.1.1/Contents/Home",
            )
        } else if cfg!(target_arch = "aarch64") {
            (
                "https://github.com/bell-sw/LibericaNIK/releases/download/24.1.1+1-23.0.1+13/bellsoft-liberica-vm-openjdk23.0.1+13-24.1.1+1-macos-aarch64.tar.gz",
                "bellsoft-liberica-vm-openjdk23-24.1.1/Contents/Home",
            )
        } else {
            panic!("Unsupported macos architecture ");
        };
        (url, "tar.gz", dir)
    } else {
        let url = if cfg!(target_arch = "x86_64") {
            "https://github.com/graalvm/graalvm-ce-builds/releases/download/jdk-23.0.1/graalvm-community-jdk-23.0.1_linux-x64_bin.tar.gz"
        } else if cfg!(target_arch = "aarch64") {
            "https://github.com/graalvm/graalvm-ce-builds/releases/download/jdk-23.0.1/graalvm-community-jdk-23.0.1_linux-aarch64_bin.tar.gz"
        } else {
            panic!("Unsupported linux architecture");
        };
        (url, "tar.gz", "graalvm-community-openjdk-23.0.1+11.1")
    };

    let graalvm_home = install_dir.join(main_dir);

    // Download and GraalVM CE
    if !graalvm_home.exists() {
        fs::create_dir_all(install_dir).unwrap();
        let archive_path = install_dir
            .join("graalvm-ce-archive")
            .with_extension(archive_ext);

        // Download the GraalVM archive file if it was not downloaded before
        if !archive_path.exists() {
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(60 * 5)) // 5 minutes
                .build()
                .unwrap();
            let response = client.get(base_url).send().unwrap();
            // copy the resp bytes to a buffer first. This will prevent creating a corrupt archive
            // in case of a download error
            let mut buffer: Vec<u8> = vec![];
            io::copy(
                &mut response
                    .bytes()
                    .unwrap_or_else(|_| panic!("Failed to download GraalVM JDK from {}", base_url))
                    .as_ref(),
                &mut buffer,
            )
            .unwrap();
            //let mut out = fs::File::create(&archive_path).unwrap();
            //out.write_all(&buffer).unwrap();
            fs::write(&archive_path, &buffer).expect("Failed to write archive file");
        }
        println!("Progress: GraalVM JDK downloaded");

        // Extract the archive file
        if archive_path.exists() {
            println!(
                "Progress: extracting the GraalVM JDK at [{}]",
                archive_path.display()
            );

            if cfg!(target_os = "windows") {
                let archive_file = fs::File::open(&archive_path).unwrap();
                let mut archive =
                    zip::ZipArchive::new(std::io::BufReader::new(archive_file)).unwrap();

                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    let outpath = install_dir.join(file.name());

                    if file.is_dir() {
                        fs::create_dir_all(&outpath).unwrap();
                    } else {
                        if let Some(parent) = outpath.parent() {
                            if !parent.exists() {
                                fs::create_dir_all(parent).unwrap();
                            }
                        }
                        let mut outfile = fs::File::create(&outpath).unwrap();
                        io::copy(&mut file, &mut outfile).unwrap();
                    }
                }
            } else {
                let tar_gz_file = fs::File::open(&archive_path).unwrap();
                let tar = flate2::read::GzDecoder::new(tar_gz_file);
                let mut archive = tar::Archive::new(tar);
                archive.unpack(install_dir).unwrap();
            }

            println!("Progress: GraalVM JDK extracted");
        } else {
            panic!("Failed to download GraalVM JDK from {}", base_url);
        }
    }

    install_dir.join(main_dir)
}
