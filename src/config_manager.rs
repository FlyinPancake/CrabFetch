use std::{env, fmt::{Debug, Display}, fs::{self, File}, io::Write, path::{Path, PathBuf}};

use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;
#[cfg(feature = "jsonschema")]
use schemars::JsonSchema;

use crate::{ascii::AsciiConfiguration, battery::BatteryConfiguration, cpu::CPUConfiguration, datetime::DateTimeConfiguration, desktop::DesktopConfiguration, displays::DisplayConfiguration, editor::EditorConfiguration, formatter::CrabFetchColor, gpu::GPUConfiguration, host::HostConfiguration, hostname::HostnameConfiguration, initsys::InitSystemConfiguration, locale::LocaleConfiguration, memory::MemoryConfiguration, modules::localip::LocalIPConfiguration, mounts::MountConfiguration, os::OSConfiguration, packages::PackagesConfiguration, processes::ProcessesConfiguration, shell::ShellConfiguration, swap::SwapConfiguration, terminal::TerminalConfiguration, uptime::UptimeConfiguration, util};
#[cfg(feature = "player")]
use crate::player::PlayerConfiguration;


#[derive(Deserialize)]
#[cfg_attr(feature = "jsonschema", derive(JsonSchema))]
pub struct Configuration {
    pub modules: Vec<String>,
    pub unknown_as_text: bool,
    pub separator: String,
    pub title_color: CrabFetchColor,
    pub title_bold: bool,
    pub title_italic: bool,
    pub decimal_places: u32,
    pub inline_values: bool,
    pub underline_character: char,
    pub color_character: String,
    pub color_margin: u8,
    pub color_use_background: bool,
    pub use_os_color: bool,
    pub segment_top: String,
    pub segment_bottom: String,
    pub progress_left_border: String,
    pub progress_right_border: String,
    pub progress_progress: String,
    pub progress_empty: String,
    pub progress_target_length: u8,
    pub percentage_color_thresholds: Vec<String>,
    pub use_ibis: bool,
    pub use_version_checksums: bool,
    pub suppress_errors: bool,

    pub ascii: AsciiConfiguration,

    pub hostname: HostnameConfiguration,
    pub cpu: CPUConfiguration,
    pub gpu: GPUConfiguration,
    pub memory: MemoryConfiguration,
    pub swap: SwapConfiguration,
    pub mounts: MountConfiguration,
    pub host: HostConfiguration,
    pub displays: DisplayConfiguration,
    pub os: OSConfiguration,
    pub packages: PackagesConfiguration,
    pub desktop: DesktopConfiguration,
    pub terminal: TerminalConfiguration,
    pub shell: ShellConfiguration,
    pub uptime: UptimeConfiguration,
    pub battery: BatteryConfiguration,
    pub locale: LocaleConfiguration,
    #[cfg(feature = "player")]
    pub player: PlayerConfiguration,
    pub editor: EditorConfiguration,
    pub initsys: InitSystemConfiguration,
    pub processes: ProcessesConfiguration,
    pub datetime: DateTimeConfiguration,
    pub localip: LocalIPConfiguration
}

// Config Error 
pub struct ConfigurationError {
    config_file: String,
    message: String
}
impl ConfigurationError {
    pub fn new(file_path: Option<String>, message: String) -> ConfigurationError {
        ConfigurationError {
            config_file: file_path.unwrap_or("Unknown".to_string()),
            message
        }
    }
}
impl Display for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse from file '{}': {}", self.config_file, self.message)
    }
}
impl Debug for ConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} failed to parse: {}", self.config_file, self.message)
    }
}

pub fn parse(location_override: &Option<String>, module_override: &Option<String>, ignore_file: &bool) -> Result<Configuration, ConfigurationError> {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    let mut config_path_str: Option<String> = None;
    if !ignore_file {
        if location_override.is_some() {
            config_path_str = Some(shellexpand::tilde(&location_override.clone().unwrap()).to_string());
            let config_path_str: String = config_path_str.as_ref().unwrap().to_string();
            // Config won't be happy unless it ends with .toml
            if !config_path_str.ends_with(".toml") {
                return Err(ConfigurationError::new(Some(config_path_str), "Config path MUST end with '.toml'".to_string()));
            }

            // Verify it exists
            let path: &Path = Path::new(&config_path_str);
            if !path.exists() {
                return Err(ConfigurationError::new(Some(config_path_str), "Unable to find config file.".to_string()));
            }
        } else {
            // Find the config path
            config_path_str = find_file_in_config_dir("config.toml").map(|x| x.display().to_string());
        }

        if config_path_str.is_some() {
            builder = builder.add_source(config::File::with_name(config_path_str.as_ref().unwrap()).required(false));
        }
    }
    // Set the defaults here
    // General
    builder = builder.set_default("modules", vec![
        "hostname".to_string(),
        "underline:16".to_string(),

        "cpu".to_string(),
        "gpu".to_string(),
        "memory".to_string(),
        "swap".to_string(),
        "mounts".to_string(),
        "host".to_string(),
        "displays".to_string(),

        "os".to_string(),
        "packages".to_string(),
        "desktop".to_string(),
        "terminal".to_string(),
        "shell".to_string(),
        "editor".to_string(),
        "uptime".to_string(),
        "locale".to_string(),
        "player".to_string(),
        "initsys".to_string(),
        "processes".to_string(),
        "battery".to_string(),
        "localip".to_string(),

        "space".to_string(),
        "colors".to_string(),
        "bright_colors".to_string(),
    ]).unwrap();

    // Android only module
    #[cfg(feature = "android")]
    if env::consts::OS == "android" {
        builder = builder.set_default("modules", vec![
            "hostname".to_string(),
            "underline:16".to_string(),

            "cpu".to_string(),
            "memory".to_string(),
            "swap".to_string(),
            "mounts".to_string(),
            "host".to_string(),

            "os".to_string(),
            "packages".to_string(),
            "terminal".to_string(),
            "shell".to_string(),
            "editor".to_string(),
            "uptime".to_string(),
            "locale".to_string(),

            "space".to_string(),
            "colors".to_string(),
            "bright_colors".to_string(),
        ]).unwrap();
    }
    builder = builder.set_default("unknown_as_text", false).unwrap();

    builder = builder.set_default("separator", " > ").unwrap();
    builder = builder.set_default("title_color", "bright_magenta").unwrap();
    builder = builder.set_default("title_bold", true).unwrap();
    builder = builder.set_default("title_italic", false).unwrap();

    builder = builder.set_default("decimal_places", 2).unwrap();
    builder = builder.set_default("inline_values", false).unwrap();
    builder = builder.set_default("underline_character", "―").unwrap();
    builder = builder.set_default("color_character", "   ").unwrap();
    builder = builder.set_default("color_margin", 0).unwrap();
    builder = builder.set_default("color_use_background", true).unwrap();

    builder = builder.set_default("use_os_color", true).unwrap();

    builder = builder.set_default("segment_top", "{color-white}[======------{color-brightmagenta} {name} {color-white}------======]").unwrap();
    builder = builder.set_default("segment_bottom", "{color-white}[======------{color-brightmagenta} {name_sized_gap} {color-white}------======]").unwrap();

    builder = builder.set_default("progress_left_border", "[").unwrap();
    builder = builder.set_default("progress_right_border", "]").unwrap();
    builder = builder.set_default("progress_progress", "=").unwrap();
    builder = builder.set_default("progress_empty", " ").unwrap();
    builder = builder.set_default("progress_target_length", 20).unwrap();

    builder = builder.set_default("use_ibis", false).unwrap();
    builder = builder.set_default("use_version_checksums", false).unwrap();
    builder = builder.set_default("suppress_errors", true).unwrap();

    builder = builder.set_default("percentage_color_thresholds", vec!["75:brightgreen", "85:brightyellow", "90:brightred"]).unwrap();

    // ASCII
    builder = builder.set_default("ascii.display", true).unwrap();
    builder = builder.set_default("ascii.colors", vec!["bright_magenta"]).unwrap();
    builder = builder.set_default("ascii.margin", 4).unwrap();
    builder = builder.set_default("ascii.side", "left").unwrap();

    // Modules
    builder = builder.set_default("hostname.title", "").unwrap();
    builder = builder.set_default("hostname.format", "{color-title}{username}{color-white}@{color-title}{hostname}").unwrap();

    builder = builder.set_default("cpu.title", "CPU").unwrap();
    builder = builder.set_default("cpu.format", "{name} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz").unwrap();
    builder = builder.set_default("cpu.remove_trailing_processor", true).unwrap();

    builder = builder.set_default("gpu.amd_accuracy", true).unwrap();
    builder = builder.set_default("gpu.ignore_disabled_gpus", true).unwrap();
    builder = builder.set_default("gpu.title", "GPU").unwrap();
    builder = builder.set_default("gpu.format", "{vendor} {model} ({vram})").unwrap();

    builder = builder.set_default("memory.title", "Memory").unwrap();
    builder = builder.set_default("memory.format", "{used} / {max} ({percent})").unwrap();

    builder = builder.set_default("swap.title", "Swap").unwrap();
    builder = builder.set_default("swap.format", "{used} / {total} ({percent})").unwrap();

    builder = builder.set_default("mounts.title", "Disk ({mount})").unwrap();
    builder = builder.set_default("mounts.format", "{space_used} used of {space_total} ({percent}) [{filesystem}]").unwrap();
    builder = builder.set_default("mounts.ignore", vec![""]).unwrap();

    builder = builder.set_default("host.title", "Host").unwrap();
    builder = builder.set_default("host.format", "{host} ({chassis})").unwrap();
    builder = builder.set_default("host.newline_chassis", false).unwrap();
    builder = builder.set_default("host.chassis_title", "Chassis").unwrap();
    builder = builder.set_default("host.chassis_format", "{chassis}").unwrap();

    builder = builder.set_default("displays.title", "Display ({make} {model})").unwrap();
    builder = builder.set_default("displays.format", "{width}x{height} @ {refresh_rate}Hz ({name})").unwrap();
    builder = builder.set_default("displays.scale_size", false).unwrap();

    builder = builder.set_default("os.title", "Operating System").unwrap();
    builder = builder.set_default("os.format", "{distro} ({kernel})").unwrap();
    builder = builder.set_default("os.newline_kernel", false).unwrap();
    builder = builder.set_default("os.kernel_title", "Kernel").unwrap();
    builder = builder.set_default("os.kernel_format", "Linux {kernel}").unwrap();


    builder = builder.set_default("packages.title", "Packages").unwrap();
    builder = builder.set_default("packages.format", "{count} ({manager})").unwrap();
    builder = builder.set_default("packages.ignore", Vec::<String>::new()).unwrap();

    builder = builder.set_default("desktop.title", "Desktop").unwrap();
    builder = builder.set_default("desktop.format", "{desktop} ({display_type})").unwrap();

    builder = builder.set_default("terminal.title", "Terminal").unwrap();
    builder = builder.set_default("terminal.format", "{name} {version}").unwrap();

    builder = builder.set_default("shell.title", "Shell").unwrap();
    builder = builder.set_default("shell.format", "{name} {version}").unwrap();
    builder = builder.set_default("shell.show_default_shell", "false").unwrap();

    builder = builder.set_default("uptime.title", "Uptime").unwrap();

    builder = builder.set_default("battery.title", "Battery {index}").unwrap();
    builder = builder.set_default("battery.format", "{percentage}%").unwrap();

    builder = builder.set_default("editor.title", "Editor").unwrap();
    builder = builder.set_default("editor.format", "{name} {version}").unwrap();
    builder = builder.set_default("editor.fancy", true).unwrap();

    builder = builder.set_default("locale.title", "Locale").unwrap();
    builder = builder.set_default("locale.format", "{language} ({encoding})").unwrap();

    builder = builder.set_default("player.title", "Player ({player})").unwrap();
    builder = builder.set_default("player.format", "{track} by {track_artists} ({album}) [{status}]").unwrap();
    builder = builder.set_default("player.ignore", Vec::<String>::new()).unwrap();

    builder = builder.set_default("initsys.title", "Init System").unwrap();
    builder = builder.set_default("initsys.format", "{name} {version}").unwrap();

    builder = builder.set_default("processes.title", "Total Processes").unwrap();

    builder = builder.set_default("datetime.title", "Date/Time").unwrap();
    builder = builder.set_default("datetime.format", "%H:%M:%S on %e %B %G").unwrap();

    builder = builder.set_default("localip.title", "Local IP ({interface})").unwrap();
    builder = builder.set_default("localip.format", "{addr}").unwrap();

    // Check for any module overrides
    if module_override.is_some() {
        let module_override: String = module_override.clone().unwrap();
        builder = builder.set_override("modules", module_override.split(',').collect::<Vec<&str>>()).unwrap();
    }

    // Now stop.
    let config: Config = match builder.build() {
        Ok(r) => r,
        Err(e) => return Err(ConfigurationError::new(config_path_str, e.to_string())),
    };

    let deserialized: Configuration = match config.try_deserialize::<Configuration>() {
        Ok(r) => r,
        Err(e) => return Err(ConfigurationError::new(config_path_str, e.to_string())),
    };

    Ok(deserialized)
}

fn find_file_in_config_dir(path: &str) -> Option<PathBuf> {
    // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut temp_var_to_shut_up_the_borrow_checker: String;
    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        temp_var_to_shut_up_the_borrow_checker = config_home;
        temp_var_to_shut_up_the_borrow_checker.push_str("/CrabFetch/");
        temp_var_to_shut_up_the_borrow_checker.push_str(path);
        paths.push(PathBuf::from(temp_var_to_shut_up_the_borrow_checker));
    }
    let mut temp_var_to_shut_up_the_borrow_checker: String;
    if let Ok(user_home) = env::var("HOME") {
        temp_var_to_shut_up_the_borrow_checker = user_home;
        temp_var_to_shut_up_the_borrow_checker.push_str("/.config/CrabFetch/");
        temp_var_to_shut_up_the_borrow_checker.push_str(path);
        paths.push(PathBuf::from(temp_var_to_shut_up_the_borrow_checker));
    }

    util::find_first_pathbuf_exists(paths)
}

pub fn check_for_ascii_override() -> Option<String> {
    let path: PathBuf = match find_file_in_config_dir("ascii") {
        Some(r) => r,
        None => return None
    };
    if !path.exists() {
        return None;
    }

    match util::file_read(&path) {
        Ok(r) => Some(r),
        Err(_) => None,
    }
}

pub fn generate_config_file(location_override: Option<String>) {
    let path: String;
    if location_override.is_some() {
        path = shellexpand::tilde(&location_override.unwrap()).to_string();
        // Config won't be happy unless it ends with .toml
        if !path.ends_with(".toml") {
            // Simply crash, to avoid confusing the user as to why the default config is being used
            // instead of their custom one.
            panic!("Config path must end with '.toml'");
        }
    } else {
        // Find the config path
        // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
        path = match env::var("XDG_CONFIG_HOME") {
            Ok(mut r) => {
                r.push_str("/CrabFetch/config.toml");
                r
            }
            Err(_) => {
                // Let's try the home directory
                let mut home_dir: String = match env::var("HOME") {
                    Ok(r) => r,
                    Err(e) => panic!("Unable to find suitable config folder; {}", e)
                };
                home_dir.push_str("/.config/CrabFetch/config.toml");
                home_dir
            }
        };
    }
    let config_path: &Path = Path::new(&path);

    if config_path.exists() {
        panic!("Path already exists: {}", config_path.display());
    }
    match fs::create_dir_all(config_path.parent().unwrap()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to create directory: {}", e),
    };

    let mut file: File = match File::create(config_path) {
        Ok(r) => r,
        Err(e) => panic!("Unable to create file; {}", e),
    };
    match file.write_all(DEFAULT_CONFIG_CONTENTS.as_bytes()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to write to file; {}", e),
    };
    println!("Created default config file at {}", path);
}

mod tests {
    // Test configs get created correctly, in the correct place and that the TOML is valid
    #[test]
    fn create_config() {
        use std::{fs, path::Path, io::Error};

        let location: String = "/tmp/crabfetch_test_config.toml".to_string();
        crate::config_manager::generate_config_file(Some(location.clone()));
        assert!(Path::new(&location).exists());

        // Attempt to parse it
        let parse = crate::config_manager::parse(&Some(location.clone()), &None, &false);
        assert!(crate::config_manager::parse(&Some(location.clone()), &None, &false).is_ok(), "{:?}", parse.err());
        
        // Finally, we remove the tmp config file 
        let removed: Result<(), Error> = fs::remove_file(location);
        assert!(removed.is_ok()); // Asserting this cus if the file fails to remove it's likely cus it never existed
    }
    
    // Tests that the default-config.toml file is the same as the DEFAULT_CONFIG_CONTENTS string in
    // here 
    // In case anyone's wondering why they're separated; it's so that package maintainers or people
    // who want a copy of the default config without re-genning it can have it without digging in
    // CrabFetch's source code
    // This test's just to make sure I keep it up to date and don't forget to update one or the
    // other
    #[test]
    fn config_is_consistent() {
        use std::{path::{PathBuf, Path}, fs::File, io::Read};
        let mut cargo_loc = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        cargo_loc.push("default-config.toml");
        assert!(Path::new(&cargo_loc).exists());

        let mut file: File = File::open(cargo_loc).unwrap();
        let mut file_contents: String = String::new();
        let _ = file.read_to_string(&mut file_contents);
        // File saving will sometimes add new lines in different places than the rust ver, so I
        // don't bother checking it as it just causes problems
        file_contents = file_contents.replace('\n', "");
        let comparing: &str = &crate::config_manager::DEFAULT_CONFIG_CONTENTS.replace('\n', "");

        assert_eq!(file_contents, comparing);
    }
}

// The default config, stored so that it can be written
const DEFAULT_CONFIG_CONTENTS: &str = r#"# For more in-depth configuration documentation, please view https://github.com/LivacoNew/CrabFetch/wiki


# The modules to display and in what order.
# Again for a full list of modules, go to the documentation above.
modules = [
    "hostname",
    "underline:16",

    "cpu",
    "gpu",
    "memory",
    "swap",
    "mounts",
    "host",
    "displays",

    "os",
    "packages",
    "desktop",
    "terminal",
    "shell",
    "editor",
    "uptime",
    "locale",
    "player",
    "initsys",
    "processes",
    "battery",
    "localip",

    "space",
    "colors",
    "bright_colors"
]

# Whether to treat unknown modules as a raw text output, allowing you to use custom strings n stuff.
# Yes, these support color placeholders.
unknown_as_text = false

# The default separator between a modules title and it's value
separator = " > "
# The default color of a modules title
# Can be; black, red, green, yellow, blue, magenta, cyan, white
# All of these can be prefixed with "bright_" to be lighter versions, e.g bright_red
# REQUIRES use_os_color TO BE OFF
title_color = "bright_magenta"
# Whether to bold/italic the title by default too
title_bold = true
title_italic = false

# The default decimal places to provide in a module
decimal_places = 2

# Whether to have all module values as inline, e.g; https://i.imgur.com/UNyq2zj.png
# To add padding use the "separator" and add some spaces
inline_values = false

# The character to use in the underline module
underline_character = '―'

# The character for each color in the colors module
color_character = "   "
# Margin between each character
color_margin = 0
# And if to set the color to the background instead of on the character
color_use_background = true

# Whether to use the distro's preferred color for the title and ASCII displays
# Disable to use custom default title colors, or custom ASCII colors
use_os_color = true

# Format of segments
# Segments can be defined in the modules array
segment_top = "{color-white}[======------{color-brightmagenta} {name} {color-white}------======]"
segment_bottom = "{color-white}[======------{color-brightmagenta} {name_sized_gap} {color-white}------======]"

# Formatting characters used in progress bars 
progress_left_border = '['
progress_right_border = ']'
progress_progress = '='
progress_empty = ' '
# The target length of the progress bar
progress_target_length = 20

# Whether to use 'ibibytes opposed to 'gabytes 
# E.g use Gibibytes (GiB) opposed to Gigabytes (GB)
use_ibis = false

# Whether to use known checksums to attempt to find the version of some stuff e.g terminal/shell/editor
# Disabled by default as it was seen as "too cheaty"
# If your benchmarking, disable it as well. If your a end user, you likely won't care if it's on or not.
use_version_checksums = false

# Whether to supress any errors that come or not
suppress_errors = true

# Percentage coloring thresholds 
# Empty this section to make it not color 
# Values are in the format of "{percentage}:{color}"
percentage_color_thresholds = [
    "75:brightgreen",
    "85:brightyellow",
    "90:brightred"
]


[ascii]
# If to display the ASCII distro art or not
display = true

# The colors to render the ASCII in
# This array can be as long as the actual ASCII. Each entry represents the color at a certain %
# E.g ["red", "green"] would render the top half as red and the bottom half as green.
# ["yellow", "blue", "magenta"] would render 33.33% as yellow, then blue, than magenta.
#
# REQUIRES use_os_color TO BE OFF
colors = ["bright_magenta"]

# The amount of space to put between the ASCII and the info
margin = 4

# Which side of the modules to display the ASCII on, left, right, top or bottom
side = "left"




# Below here is the actual modules
# Refer to the wiki for any module-specific parameters or hidden parameters
# Also remember that you can override some stuff on these, e.g the title formatting. Again check the wiki.

[hostname]
# Placeholders;
# {hostname} -> The hostname
# {username} -> The username of the current user
title = ""
format = "{color-title}{username}{color-white}@{color-title}{hostname}"


[cpu]
# Placeholders;
# {name} -> The name of the cpu.
# {core_count} -> The number of cores.
# {thread_count} -> The number of threads.
# {current_clock_mhz} -> The current clock speed, in MHz.
# {current_clock_ghz} -> The current clock speed, in GHz.
# {max_clock_mhz} -> The maximum clock speed, in MHz.
# {max_clock_ghz} -> The maximum clock speed, in GHz.
# {arch} -> The architecture of your CPU.
title = "CPU"
format = "{name} {arch} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz"

# Whether to attempt to remove any trailing "x-Core Processor" left in the branding name by the manufacturer
# May not be perfect, disable and report an issue if output looks odd.
remove_trailing_processor = true


[gpu]
# Whether to try to search a separate AMD specific file to try to improve accuracy on AMD GPU's 
amd_accuracy = true

# Ignore any GPU's that are marked as "disabled" by Linux
ignore_disabled_gpus = true


# Placeholders;
# - {index} -> The index of the GPU, only useful if you have more than one GPU.
# - {vendor} -> The vendor of the GPU, e.g AMD
# - {model} -> The model of the GPU, e.g Radeon RX 7800XT
# - {vram} -> The total memory of the GPU.
title = "GPU"
format = "{vendor} {model} ({vram})"


[memory]
# Placeholders;
# {used} -> The currently in-use memory.
# {max} -> The maximum total memory.
# {bar} -> A progress bar representing the total space available/taken.
# {percent} -> Percentage of memory used
title = "Memory"
format = "{used} / {max} ({percent})"


[swap]
# Placeholders;
# {used} -> The currently used swap.
# {max} -> The maximum total swap.
# {bar} -> A progress bar representing the total space available/taken.
# {percent} -> Percentage of swap used
title = "Swap"
format = "{used} / {total} ({percent})"


[mounts]
# This module is a multi-line module, each mount has it's own line in the output. 
# Placeholders;
# {device} -> Device, e.g /dev/sda
# {mount} -> The mount point, e.g /home
# {space_used} -> The space used.
# {space_avail} -> The space available.
# {space_total} -> The total space.
# {filesystem} -> The filesystem running on that mount.
# {bar} -> A progress bar representing the total space available/taken.
# {percent} -> The percentage of the disk used.
title = "Disk ({mount})"
format = "{space_used} used of {space_total} ({percent}) [{filesystem}]"

# A ignore list for any point points OR filesystems to ignore
# The entries only need to start with these to be ignored
# It's also worth noting that CrabFetch automatically ignores any non-physical device mount
ignore = []


[host]
# Placeholders;
# {host} -> The name of the host, either a motherboard name or a laptop model
# {chassis} -> The chassis type, e.g Desktop or Laptop or whatever
title = "Host"
format = "{host} ({chassis})"

# Whether to output the chassis on it's own line to remain consistent with other fetch scripts.
newline_chassis = false
# The title/format of the chassis if we are outputting on it's own line
chassis_title = "Chassis"
chassis_format = "{chassis}"


[displays]
# This module is a multi-line module, each display will have it's own line in the output.
# Placeholders;
# {make} -> The monitor's make
# {model} -> The monitor's model
# {name} -> The monitor DRM name, e.g DP-2
# {width} -> The monitor's width
# {height} -> The monitor's height
# {refresh_rate} -> The monitor's refresh rate. This won't work in x11!
title = "Display ({make} {model})"
format = "{width}x{height} @ {refresh_rate}Hz ({name})"

# Whether to scale the width/height according to the screen's scale. Only availabe on Wayland.
# **This will output wrong with fractional scaling**, as the library we use to interact with Wayland doesn't support fractional scaling yet.
scale_size = false


[os]
# Placeholders;
# {distro} -> The distro name
# {kernel} -> The kernel version
title = "Operating System"
format = "{distro} ({kernel})"

# Display the kernel version on a newline and if so, what format to use 
newline_kernel = false
kernel_title = "Kernel"
kernel_format = "Linux {kernel}"


[packages]
# This format is for each entry, with all entries being combined into a single string separated by a comma. Placeholders;
# {manager} -> The name of the manager
# {count} -> The amount of packages that manager reports
title = "Packages"
format = "{count} ({manager})"

# List of package managers to ignore, for whatever reason you choose to
ignore = []


[desktop]
# Placeholders;
# {desktop} -> The name of the desktop
# {display_type} -> The type of display server, aka x11 or wayland.
title = "Desktop"
format = "{desktop} ({display_type})"


[terminal]
# Placeholders;
# {name} -> The name of the terminal, e.g kitty
# {path} -> The path of the terminal, e.g /usr/bin/kitty
# {version} -> The version of the terminal
title = "Terminal"
format = "{name} {version}"


[shell]
# Placeholders;
# {name} -> The name of the shell, e.g zsh
# {path} -> The path of the shell, e.g /usr/bin/zsh
# {version} -> The version of the shell.
title = "Shell"
format = "{name} {version}"

# Whether to show your default shell, instead of your current shell.
show_default_shell = false


[uptime]
title = "Uptime"


[editor]
# Placeholders;
# {name} -> The name of the editor
# {path} -> The path the editor is at
# {version} -> The version of the editor.
title = "Editor"
format = "{name} {version}"

# Whether to turn the name into a "fancy" variant. E.g "nvim" gets turned into "NeoVim"
fancy = true


[locale]
# Placeholders;
# {language} - The selected language
# {encoding} - The encoding selected, most likely UTF-8
title = "Locale"
format = "{language} ({encoding})"


[player]
# This is a multi-line module, each player detected will have it's own line in the output
# Placeholders;
# {player} -> The player currently playing
# {track} - The name of the track
# {album} - The name of the album
# {track_artists} - The names of all track artists
# {album_artists} - The names of all album artists
# {status} - The status of the player, AKA if it's playing or not.
title = "Player ({player})"
format = "{track} by {track_artists} ({album}) [{status}]"

# Any music players to ignore
# These must be valid MPRIS player strings. You can find them by running something like `playerctl --list-all`
ignore = []


[battery]
# Placeholders;
# {index} -> The batterys index
# {percentage} -> The battery percentage
# {bar} -> A progeress bar representing how full the battery is
title = "Battery {index}"
format = "{percentage}%"


[initsys]
# Placeholders;
# {name} -> The name of the init system
# {path} -> The path to the init system binary
# {version} -> The version of the init system
title = "Init System"
format = "{name} {version}"


[processes]
title = "Total Processes"


[datetime]
title = "Date Time"
# Available placeholders; https://docs.rs/chrono/latest/chrono/format/strftime/index.html#specifiers
# CrabFetch wiki page coming soon for it instead (tm)
format = "%H:%M:%S on %e %B %G"

[localip]
# This is a multi-line module, each IP/interface detected will have it's own line in the output
# Placeholders;
# {interface} -> The name of the interface, along with if it's IPV4 or IPV6
# {addr} -> The IP address
title = "Local IP ({interface})"
format = "{addr}"



# You've reached the end! Congrats, have a muffin :)"#;
