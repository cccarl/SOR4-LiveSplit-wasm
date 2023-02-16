use std::collections::HashMap;
use spinning_top::{const_spinlock, Spinlock};
use asr::{Process, watcher::{Watcher, Pair}, time::Duration};
use bytemuck::Pod;
use widestring::U16CStr;
use once_cell::sync::Lazy;

// for V07-s r13648
// TODO: multiple versions abstraction / implementation
const SUBMENUS_OPEN_PATH: [u64; 4] = [0x014B_FAB0, 0x0, 0x78, 0x28];
const CURR_SECTION_FRAMES: [u64; 4] = [0x014BFE38, 0x10, 0xA8, 0x38];
const TOTAL_FRAME_COUNT: [u64; 5] = [0x014BFE38, 0x0, 0x78, 0x10, 0x2C];
const TOTAL_FRAME_COUNT_SURVIVAL: [u64; 5] = [0x014BFE38, 0x0, 0x78, 0x10, 0x14];
const CURRENT_MUSIC: [u64; 5] = [0x014BFE30, 0x0, 0x70, 0x28, 0xC];
const CURRENT_LVL: [u64; 6] = [0x014BFE38, 0x0, 0x50, 0x18, 0x108, 0x3E];


fn read_value<T: std::fmt::Display + Pod>(process: &Process, main_module_addr: asr::Address, pointer_path: &[u64], watcher: &mut Watcher<T>, var_key: &str) -> Option<Pair<T>> {

    match process.read_pointer_path64::<T>(main_module_addr.0, pointer_path) {
        Ok(mem_value) => {
            asr::timer::set_variable(var_key, &format!("{mem_value}"));
            //asr::print_message(&format!("{}", mem_value));
            Some(*watcher.update_infallible(mem_value))
        },
        Err(_) => {
            None
        }
    }

}

fn read_value_string(process: &Process, main_module_addr: asr::Address, pointer_path: &[u64], var_key: &str) -> Option<String> {

    let buf = match process.read_pointer_path64::<[u16; 100]>(main_module_addr.0, pointer_path) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    let cstr = U16CStr::from_slice_truncate(&buf).unwrap();

    let parsed_string = cstr.to_string().unwrap_or("".to_string());

    asr::timer::set_variable(var_key, &parsed_string);

    Some(parsed_string)

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

struct Watchers {
    submenus_open: Watcher<i32>,
    current_section_frames: Watcher<i32>,
    acumm_frames: Watcher<i32>,
    acumm_frames_survival: Watcher<i32>,
}

struct MemoryValues {
    submenus_open: Pair<i32>,
    current_section_frames: Pair<i32>,
    accum_frames: Pair<i32>,
    accum_frames_survival: Pair<i32>,
    current_lvl: String,
    current_music: String,
}

struct GameTime {
    seconds: f64,
}

impl GameTime {
    // LS usually reads garbage for a moment on loading screens so this ensures that the new game time to be set is a reasonable value
    fn calculate_game_time(&mut self, current_section_frames: i32, accum_frames: i32) {
        let new_igt: f64 = (current_section_frames + accum_frames) as f64 / 60.0;

        // do not update igt if the new time is lower or it did a huge jump forwards, also reset to 0 and jump from a 0 to the correct igt
        if (new_igt > self.seconds && new_igt < self.seconds + 10.0) || new_igt == 0.0 || self.seconds == 0.0 {
            self.seconds = new_igt
        }
    }
}

#[derive(PartialEq)]
enum GameMode {
    Normal,
    Survival,
}

struct Setting<'a> {
    key: &'a str,
    description: &'a str,
    default_value: bool,
}

struct State {
    process_info: Option<ProcessInfo>,
    watchers: Watchers,
    values: MemoryValues,
    started_up: bool,
    igt: GameTime,
    game_mode: GameMode,
    settings: Lazy<HashMap<String, bool>>,
    last_split: String,
}

impl State {

    fn startup(&mut self) {
        asr::set_tick_rate(10.0);
        asr::print_message("Started up!!!!!");

        // key, description, default value
        let settings_data = vec![
            Setting {key: "splits_stage1_1", description: "Streets", default_value: false},
            Setting {key: "splits_stage1_2", description: "Sewers", default_value: false},
            Setting {key: "splits_stage1_3", description: "Diva", default_value: true},
            Setting {key: "splits_stage2_1", description: "Jail", default_value: false},
            Setting {key: "splits_stage2_2", description: "HQ", default_value: false},
            Setting {key: "splits_stage2_3", description: "Commissioner", default_value: true},
            Setting {key: "splits_stage3_1a", description: "Outside", default_value: false},
            Setting {key: "splits_stage3_1b", description: "Inside", default_value: false},
            Setting {key: "splits_stage3_1c", description: "Hallway", default_value: false},
            Setting {key: "splits_stage3_2", description: "Nora", default_value: true},
            Setting {key: "splits_stage4_1", description: "Pier", default_value: false},
            Setting {key: "Music_Level04!BOSS", description: "Estel Start", default_value: false},
            Setting {key: "splits_stage4_2", description: "Estel", default_value: true},
            Setting {key: "splits_stage5_1", description: "Underground", default_value: false},
            Setting {key: "splits_stage5_2", description: "Bar", default_value: false},
            Setting {key: "splits_stage5_3", description: "Barbon", default_value: true},
            Setting {key: "splits_stage6_1", description: "Streets", default_value: false},
            Setting {key: "splits_stage6_2a", description: "Dojo - Galsia Room", default_value: false},
            Setting {key: "splits_stage6_2b", description: "Dojo - Donovan Room", default_value: false},
            Setting {key: "splits_stage6_2c", description: "Dojo - Pheasant Room", default_value: false},
            Setting {key: "splits_stage6_3", description: "Shiva", default_value: true},
            Setting {key: "splits_Music_Level07!BOSS", description: "Estel Start", default_value: false},
            Setting {key: "splits_stage7_1", description: "Estel", default_value: true},
            Setting {key: "splits_stage8_1", description: "Gallery", default_value: false},
            Setting {key: "splits_stage8_2", description: "Beyo and Riha", default_value: true},
            Setting {key: "splits_stage9_1", description: "Sauna", default_value: false},
            Setting {key: "splits_stage9_2", description: "Elevator", default_value: false},
            Setting {key: "splits_stage9_3", description: "Max", default_value: true},
            Setting {key: "splits_stage10_1a", description: "Rooftops - Arrival", default_value: false},
            Setting {key: "splits_stage10_1b", description: "Rooftops - Advance", default_value: false},
            Setting {key: "splits_stage10_1c", description: "Rooftops - Wrecking Balls", default_value: false},
            Setting {key: "splits_stage10_3", description: "DJ K-Washi", default_value: true},
            Setting {key: "splits_stage11_1", description: "Platform", default_value: false},
            Setting {key: "splits_stage11_2a", description: "Boarding the Airplane", default_value: false},
            Setting {key: "splits_stage11_2b", description: "Inside the Airplane", default_value: false},
            Setting {key: "splits_stage11_3", description: "Mr. Y", default_value: true},
            Setting {key: "splits_stage12_1", description: "Wreckage", default_value: false},
            Setting {key: "splits_stage12_2a", description: "Hallway", default_value: false},
            Setting {key: "splits_stage12_2b", description: "Inside Castle", default_value: false},
            Setting {key: "splits_stage12_2c", description: "Ms. Y", default_value: false},
            Setting {key: "splits_stage12_3", description: "Ms. Y, Mr. Y and Y Mecha", default_value: true},
            Setting {key: "splits_bossRush_newBoss", description: "Boss Rush - Boss Defeated", default_value: true},
            Setting {key: "splits_llenge_01_bossrun_v3", description: "Boss Rush Completed", default_value: true},
            Setting {key: "splits_survival", description: "Survival Mode - Level Complete", default_value: true},
        ];

        for setting in settings_data {
            self.settings.insert(setting.key.to_string(), asr::user_settings::add_bool(setting.key, setting.description, setting.default_value));
        }

        self.started_up = true;

    }

    fn init(&mut self) {
        asr::timer::set_variable("Submenus", "-");
        asr::timer::set_variable("Current section frames", "-");
        asr::timer::set_variable("Accumulated Frames", "-");
        asr::set_tick_rate(60.0);
        asr::print_message("ATTACHED!!!!!!!!!!");
    }

    fn refresh_mem_values(&mut self) -> Result<&str, &str> {

        let main_module_addr = match &self.process_info {
            Some(info) => info.main_module_address,
            None => return Err("Process info is not initialized")
        };

        let process = &self.process_info.as_ref().unwrap().game;

        // refresh values with watcher updates
        if let Some(value) = read_value::<i32>(process, main_module_addr , &SUBMENUS_OPEN_PATH, &mut self.watchers.submenus_open, "Submenus") {
            self.values.submenus_open = value;
        }

        if let Some(value) = read_value::<i32>(process, main_module_addr, &CURR_SECTION_FRAMES, &mut self.watchers.current_section_frames, "Current section frames") {
            self.values.current_section_frames = value;
        }

        if let Some(value) = read_value::<i32>(process, main_module_addr, &TOTAL_FRAME_COUNT, &mut self.watchers.acumm_frames, "Accumulated Frames") {
            self.values.accum_frames = value;
        }

        if let Some(value) = read_value::<i32>(process, main_module_addr, &TOTAL_FRAME_COUNT_SURVIVAL, &mut self.watchers.acumm_frames_survival, "Accumulated Frames Survival") {
            self.values.accum_frames_survival = value;
        }

        self.values.current_lvl = read_value_string(process, main_module_addr, &CURRENT_LVL, "Level Name").unwrap_or("".to_string());

        self.values.current_music = read_value_string(process, main_module_addr, &CURRENT_MUSIC, "Music Name").unwrap_or("".to_string());

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

        // refresh mem values if possible, the values will be updated using the watchers or just directly reading lvl and music strings
        if self.refresh_mem_values().is_err() {
            return;
        }

        // start condition
        // TODO: start settings
        if self.values.current_section_frames.current > 0 && self.values.current_section_frames.current < 60 && self.values.current_section_frames.changed()
        && !self.values.current_lvl.is_empty() && !self.values.current_lvl.contains("training") {
            self.last_split = String::new();
            self.igt.seconds = 0.0;
            self.game_mode = check_current_game_mode(&self.values.current_lvl);
            asr::timer::start();
        }

        // igt
        match self.game_mode {
            GameMode::Normal => self.igt.calculate_game_time(*self.values.current_section_frames, *self.values.accum_frames),
            GameMode::Survival => self.igt.calculate_game_time(*self.values.current_section_frames, *self.values.accum_frames_survival),
        }
        
        if asr::timer::state() == asr::timer::TimerState::Running {
            asr::timer::set_game_time(Duration::seconds_f64( self.igt.seconds ))
        }

        // reset condition
        if self.values.submenus_open.current == 0 && self.values.submenus_open.old == 2 {
            asr::timer::set_game_time(Duration::microseconds(0));
            asr::timer::reset();
        }

        // split conditions
        // splits - section ended and accum frames are updated
        if self.values.accum_frames.current > self.values.accum_frames.old || self.values.accum_frames_survival.current > self.values.accum_frames.old {
            let settings_key = format!("splits_{}", self.values.current_lvl);
            if (self.settings.contains_key(&settings_key) && self.settings[&settings_key] || self.settings["splits_survival"] && self.game_mode == GameMode::Survival) && self.last_split != settings_key {
                self.last_split = settings_key;
                asr::timer::split();
            }
        }

        // splits - music triggers
        if self.values.current_music == "Music_Level07!BOSS" || self.values.current_music == "Music_Level04!BOSS" {
            let settings_key = format!("splits_{}", self.values.current_music);
            if self.settings.contains_key(&settings_key) && self.settings[&settings_key] && self.last_split != settings_key {
                self.last_split = settings_key;
                asr::timer::split();
            }
        }
        
        // splits - boss rush music trigger
        if self.values.current_music.contains("BossRush") && self.values.current_music != "Music_BossRush!A00_Diva" && self.settings["splits_bossRush_newBoss"] {
            let settings_key = format!("splits_{}", self.values.current_music);
            if self.last_split != settings_key {
                self.last_split = settings_key;
                asr::timer::split();
            }
        }

    }
}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    process_info: None,
    watchers: Watchers {
        submenus_open: Watcher::new(),
        current_section_frames: Watcher::new(),
        acumm_frames: Watcher::new(),
        acumm_frames_survival: Watcher::new(),
    },
    values: MemoryValues { 
        submenus_open: Pair { old: 0, current: 0 }, 
        current_section_frames: Pair { old: 0, current: 0 },
        accum_frames: Pair { old: 0, current: 0 },
        accum_frames_survival: Pair { old: 0, current: 0 },
        current_lvl: String::new(),
        current_music: String::new(),
    },
    igt: GameTime { seconds: 0.0 },
    started_up: false,
    game_mode: GameMode::Normal,
    settings: Lazy::new(HashMap::new),
    last_split: String::new(),
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
