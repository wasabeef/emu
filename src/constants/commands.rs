//! Command line tools and executables.

/// Android SDK command-line tools
pub const ADB: &str = "adb";
pub const AVDMANAGER: &str = "avdmanager";
pub const EMULATOR: &str = "emulator";
pub const SDKMANAGER: &str = "sdkmanager";

/// iOS/macOS command-line tools
pub const XCRUN: &str = "xcrun";
pub const SIMCTL: &str = "simctl";
pub const OSASCRIPT: &str = "osascript";
pub const KILLALL: &str = "killall";

/// ADB subcommands and arguments
pub mod adb {
    pub const DEVICES: &str = "devices";
    pub const SHELL: &str = "shell";
    pub const GETPROP: &str = "getprop";
    pub const EMU: &str = "emu";
    pub const AVD: &str = "avd";
    pub const NAME: &str = "name";
    pub const KILL: &str = "kill";
    pub const LOGCAT: &str = "logcat";

    // System properties
    pub const PROP_AVD_NAME: &str = "ro.boot.qemu.avd_name";
    pub const PROP_KERNEL_AVD_NAME: &str = "ro.kernel.qemu.avd_name";
}

/// iOS Simulator subcommands
pub mod ios {
    pub const LIST: &str = "list";
    pub const DEVICES: &str = "devices";
    pub const RUNTIMES: &str = "runtimes";
    pub const BOOT: &str = "boot";
    pub const SHUTDOWN: &str = "shutdown";
    pub const ERASE: &str = "erase";
    pub const DEVTYPES: &str = "devicetypes";
    pub const CREATE: &str = "create";
    pub const DELETE: &str = "delete";
}

/// AVD Manager subcommands
pub mod avdmanager {
    pub const LIST: &str = "list";
    pub const CREATE: &str = "create";
    pub const DELETE: &str = "delete";
    pub const AVD: &str = "avd";
    pub const DEVICE: &str = "device";
    pub const TARGET: &str = "target";

    // Arguments
    pub const NAME_ARG: &str = "--name";
    pub const DEVICE_ARG: &str = "--device";
    pub const PACKAGE_ARG: &str = "--package";
    pub const TAG_ARG: &str = "--tag";
    pub const ABI_ARG: &str = "--abi";
    pub const FORCE_ARG: &str = "--force";
    pub const SKIN_ARG: &str = "--skin";
}

/// SDK Manager subcommands and arguments
pub mod sdkmanager {
    pub const LIST: &str = "--list";
    pub const VERBOSE: &str = "--verbose";
    pub const UNINSTALL: &str = "--uninstall";
    pub const INCLUDE_OBSOLETE: &str = "--include_obsolete";
}

/// Emulator arguments
pub mod emulator {
    pub const AVD_ARG: &str = "-avd";
    pub const WIPE_DATA: &str = "-wipe-data";
    pub const NO_SNAPSHOT_LOAD: &str = "-no-snapshot-load";
    pub const NO_AUDIO: &str = "-no-audio";
    pub const NO_WINDOW: &str = "-no-window";
    pub const GPU_ARG: &str = "-gpu";
    pub const MEMORY_ARG: &str = "-memory";
    pub const PARTITION_SIZE_ARG: &str = "-partition-size";
}
