// TODO
// igt survival
// splits
// settings

use spinning_top::{const_spinlock, Spinlock};
use asr::{Process, watcher::{Watcher, Pair}};
use bytemuck::Pod;
use widestring::U16CStr;

const SUBMENUS_OPEN_PATH: [u64; 4] = [0x014B_FAB0, 0x0, 0x78, 0x28];
const CURR_SECTION_FRAMES: [u64; 4] = [0x014BFE38, 0x10, 0xA8, 0x38];
const TOTAL_FRAME_COUNT: [u64; 5] = [0x014BFE38, 0x0, 0x78, 0x10, 0x2C];
const TOTAL_FRAME_COUNT_SURVIVAL: [u64; 5] = [0x014BFE38, 0x0, 0x78, 0x10, 0x14];
const CURRENT_MUSIC: [u64; 5] = [0x014BFE30, 0x0, 0x70, 0x28, 0xC];
const CURRENT_LVL: [u64; 6] = [0x014BFE38, 0x0, 0x50, 0x18, 0x108, 0x3E];


fn read_value<T: std::fmt::Display + Pod>(process: &Process, main_module_addr: asr::Address, pointer_path: &[u64], watcher: &mut Watcher<T>, var_key: &str) -> Result<Pair<T>, String> {

    match process.read_pointer_path64::<T>(main_module_addr.0, pointer_path) {
        Ok(mem_value) => {
            asr::timer::set_variable(var_key, &format!("{}", mem_value));
            //asr::print_message(&format!("{}", mem_value));
            Ok(*watcher.update_infallible(mem_value))
        },
        Err(e) => {
            asr::print_message(&format!("Could not refresh '{}' value: {:?}", var_key, e));
            Err(format!("Could not refresh '{}' value: {:?}", var_key, e))
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


struct State {
    process_info: Option<ProcessInfo>,
    watchers: Watchers,
    values: MemoryValues,
    started_up: bool,
    igt: GameTime,
    game_mode: GameMode,
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

enum GameMode {
    Normal,
    Survival,
}

impl State {

    fn startup(&mut self) {
        asr::print_message("Started up!!!!!");
        self.started_up = true;
    }

    fn init(&mut self) {
        asr::print_message("ATTACHED!!!!!!!!!!");
        asr::set_tick_rate(60.0);
        asr::timer::set_variable("Submenus", "-");
        asr::timer::set_variable("Current section frames", "-");
        asr::timer::set_variable("Accumulated Frames", "-");
    }

    fn refresh_mem_values(&mut self) -> Result<&str, &str> {

        let main_module_addr = match &self.process_info {
            Some(info) => info.main_module_address,
            None => return Err("Process info is not initialized")
        };

        let process = &self.process_info.as_ref().unwrap().game;

        // refresh values with watcher updates
        match read_value::<i32>(process, main_module_addr , &SUBMENUS_OPEN_PATH, &mut self.watchers.submenus_open, "Submenus") {
            Ok(value) => self.values.submenus_open = value,
            Err(_) => {}
        }; 
        match read_value::<i32>(process, main_module_addr, &CURR_SECTION_FRAMES, &mut self.watchers.current_section_frames, "Current section frames") {
            Ok(value) => self.values.current_section_frames = value,
            Err(_) => {}
        };
        match read_value::<i32>(process, main_module_addr, &TOTAL_FRAME_COUNT, &mut self.watchers.acumm_frames, "Accumulated Frames") {
            Ok(value) => self.values.accum_frames = value,
            Err(_) => {}
        }
        match read_value::<i32>(process, main_module_addr, &TOTAL_FRAME_COUNT_SURVIVAL, &mut self.watchers.acumm_frames_survival, "Accumulated Frames Survival") {
            Ok(value) => self.values.accum_frames_survival = value,
            Err(_) => {}
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
            if !self.process_info.is_none() {
                self.init();
            }
            // early return to never work with a None process
            return;
        }

        if !self.process_info.as_ref().unwrap().game.is_open() {
            self.process_info = None;
            return;
        }

        // refresh mem values if possible, the values will be updated using the watchers or just directly reading lvl and music strings
        if self.refresh_mem_values().is_err() {
            return;
        }


        // start condition
        // TODO: don't start in training mode, start settings
        if self.values.current_section_frames.current > 0 && self.values.current_section_frames.current < 60 && self.values.current_section_frames.changed() {
            self.igt.seconds = 0.0;
            asr::timer::start();
        }

        // reset condition
        if self.values.submenus_open.current == 0 && self.values.submenus_open.old == 2 {
            asr::timer::reset();
        }

        // igt
        self.igt.calculate_game_time(*self.values.current_section_frames, *self.values.accum_frames);
        asr::timer::set_game_time(asr::time::Duration::seconds_f64( self.igt.seconds ));


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
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
