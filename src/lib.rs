use spinning_top::{const_spinlock, Spinlock};
use asr::{Process, watcher::{Pair}, time::Duration};
use widestring::U16CStr;
use once_cell::sync::Lazy;
use settings::Settings;

mod settings;
mod memory;
mod splits;

fn update_pair_i32(variable_name: &str, new_value: i32, pair: &mut Pair<i32>) {
    asr::timer::set_variable(variable_name, &format!("{new_value}"));
    pair.old = pair.current;
    pair.current = new_value;
}

fn read_widestring_and_update_pair(process: &Process, main_module_addr: asr::Address, pair: &mut Pair<String>, pointer_path: &[u64], var_key: &str) {

    let buf = match process.read_pointer_path64::<[u16; 100]>(main_module_addr.0, pointer_path) {
        Ok(bytes) => bytes,
        Err(_) => return,
    };

    if buf.len() == 0 {
        return;
    }
    let cstr = U16CStr::from_slice_truncate(&buf).unwrap();

    let parsed_string = cstr.to_string().unwrap_or("".to_string());

    asr::timer::set_variable(var_key, &parsed_string);
    pair.old = pair.current.clone();
    pair.current = parsed_string;
    
}

fn check_current_game_mode(level_name: &String) -> GameMode {
    if level_name.contains("stage") || level_name.contains("boss") || level_name.is_empty() {
        GameMode::Normal
    }
    else {
        GameMode::Survival
    }
}

struct ProcessInfo {
    game: Process,
    main_module_address: asr::Address,
}

impl ProcessInfo {
    fn new(process: Process) -> Self {
        Self {
            main_module_address: process.get_module_address("SOR4.exe").unwrap_or(asr::Address(0)),
            game: process,
        }
    }
}

#[derive(Default)]
pub struct PointerPaths {
    submenus_open: Vec<u64>,
    current_section_frames: Vec<u64>,
    accum_frames: Vec<u64>,
    accum_frames_survival: Vec<u64>,
    current_lvl: Vec<u64>,
    current_music: Vec<u64>,
}

#[derive(Default)]
struct MemoryValues {
    submenus_open: Pair<i32>,
    current_section_frames: Pair<i32>,
    accum_frames: Pair<i32>,
    accum_frames_survival: Pair<i32>,
    current_lvl: Pair<String>,
    current_music: Pair<String>,
}

struct GameTime {
    igt: asr::time::Duration,
}

impl GameTime {
    // LS usually reads garbage for a moment on loading screens so this ensures that the new game time to be set is a reasonable value
    fn calculate_game_time(&mut self, current_section_frames: i32, accum_frames: i32) {
        let new_igt: f64 = (current_section_frames + accum_frames) as f64 / 60.0;
        let igt_in_seconds =  self.igt.as_seconds_f64();

        // do not update igt if the new time is lower or it did a huge jump forwards, also reset to 0 and jump from a 0 to the correct igt
        if (new_igt > igt_in_seconds && new_igt < igt_in_seconds + 10.0) || new_igt == 0.0 || igt_in_seconds == 0.0 {
            self.igt = asr::time_util::frame_count::<60>((current_section_frames + accum_frames) as u64);
        }

    }
}

#[derive(PartialEq)]
enum GameMode {
    Normal,
    Survival,
}

#[derive(Debug, Clone, Copy)]
pub enum Version {
    Unsupported,
    V08SR14424,
    V07SR13648,
}

struct State {
    process_info: Option<ProcessInfo>,
    values: Lazy<MemoryValues>,
    pointer_paths: Lazy<PointerPaths>,
    started_up: bool,
    igt: GameTime,
    game_mode: GameMode,
    settings: Option<Settings>,
    version: Version,
}

impl State {

    fn startup(&mut self) {
        asr::set_tick_rate(10.0);
        self.settings = Some(Settings::register());
        self.started_up = true;

    }

    fn init(&mut self) {
        asr::timer::set_variable("Submenus", "-");
        asr::timer::set_variable("Current section frames", "-");
        asr::timer::set_variable("Accumulated Frames", "-");

        let sor4_size = self.process_info.as_ref().unwrap().game.get_module_size("SOR4.exe").unwrap_or(0);

        match sor4_size {
            0x1657000 => self.version = Version::V08SR14424,
            0x1638000 => self.version = Version::V07SR13648,
            _ => {
                asr::print_message(&format!("Patch not supported. Module Memory Size: {:X}", sor4_size));
                self.version = Version::Unsupported;
            },
        }
        
        asr::timer::set_variable("Version", &format!("{:?}", self.version));

        *self.pointer_paths = memory::get_pointer_paths(self.version);

        asr::set_tick_rate(60.0);
        asr::print_message("Attached!!!");
    }

    // TODO: move to memory module
    fn refresh_mem_values(&mut self) -> Result<&str, &str> {

        // TODO: only query on init(), not every update
        let main_module_addr = match &self.process_info {
            Some(info) => info.main_module_address,
            None => return Err("Process info is not initialized")
        };

        let process = &self.process_info.as_ref().unwrap().game;

        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &self.pointer_paths.submenus_open) {
            update_pair_i32("Submenus", value, &mut self.values.submenus_open);
        }

        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &self.pointer_paths.current_section_frames) {
            update_pair_i32("Current section frames", value, &mut self.values.current_section_frames);
        }

        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &self.pointer_paths.accum_frames) {
            update_pair_i32("Accumulated Frames", value, &mut self.values.accum_frames);
        }

        if let Ok(value) = process.read_pointer_path64::<i32>(main_module_addr.0, &self.pointer_paths.accum_frames_survival) {
            update_pair_i32("Accumulated Frames Survival", value, &mut self.values.accum_frames_survival);
        }

        read_widestring_and_update_pair(process, main_module_addr, &mut self.values.current_lvl, &self.pointer_paths.current_lvl, "Level Name");

        read_widestring_and_update_pair(process, main_module_addr, &mut self.values.current_music, &self.pointer_paths.current_music, "Music Name");

        Ok("Success")
    }

    fn update(&mut self) {

        if !self.started_up {
            self.startup();
        }

        if self.process_info.is_none() {
            self.process_info = Process::attach("SOR4.exe").map(ProcessInfo::new);
            if self.process_info.is_some() {
                self.init();
            }
            // early return to never work with a None process
            return;
        }

        // if game is closed detatch and look for the game again
        if !self.process_info.as_ref().unwrap().game.is_open() {
            asr::set_tick_rate(10.0);
            self.process_info = None;
            return;
        }

        // refresh mem values if possible, i32 values are in pairs (old and current) and strings only have the current value
        if self.refresh_mem_values().is_err() {
            return;
        }
        
        // start condition
        // TODO: start settings
        if self.values.current_section_frames.current > 10 && self.values.current_section_frames.current < 60 && self.values.current_section_frames.changed()
         {
            self.igt.igt = asr::time::Duration::seconds(0);
            self.game_mode = check_current_game_mode(&self.values.current_lvl.current);
            asr::timer::start();
        }

        // igt
        match self.game_mode {
            GameMode::Normal => self.igt.calculate_game_time(*self.values.current_section_frames, *self.values.accum_frames),
            GameMode::Survival => self.igt.calculate_game_time(*self.values.current_section_frames, *self.values.accum_frames_survival),
        }
        
        if asr::timer::state() == asr::timer::TimerState::Running {
            asr::timer::set_game_time(self.igt.igt);
        }

        // reset condition
        if self.values.submenus_open.current == 0 && self.values.submenus_open.old == 2 {
            asr::timer::set_game_time(Duration::microseconds(0));
            asr::timer::reset();
        }

        // split conditions
        if self.should_split() {
            asr::timer::split();
        }

        // stops livesplits game time jitter
        asr::timer::pause_game_time();

    }
}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    process_info: None,
    values: Lazy::new(MemoryValues::default),
    pointer_paths: Lazy::new(PointerPaths::default),
    igt: GameTime { igt: asr::time::Duration::seconds(0) },
    started_up: false,
    game_mode: GameMode::Normal,
    settings: None,
    version: Version::Unsupported,
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
