use spinning_top::{const_spinlock, Spinlock};
use asr::{Process, watcher::{Watcher, Pair}};
use bytemuck;

const SUBMENUS_OPEN_PATH: [u64; 4] = [0x014B_FAB0, 0x0, 0x78, 0x28];
const CURR_SECTION_FRAMES: [u64; 4] = [0x014BFE38, 0x10, 0xA8, 0x38];
const TOTAL_FRAME_COUNT: [u64; 5] = [0x014BFE38, 0x0, 0x78, 0x10, 0x2C];

fn read_value<T: Copy + std::fmt::Display + bytemuck::Pod>(process: &Process, main_module_addr: asr::Address, pointer_path: &[u64], watcher: &mut Watcher<T>, var_key: &str) -> Result<Pair<T>, String> {

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

struct ProcessInfo {
    game: Process
}

impl ProcessInfo {
    fn new(process: Process) -> Self {
        Self {
            game: process,
        }
    }
}


struct State {
    process_info: Option<ProcessInfo>,
    watchers: Watchers,
    values: MemoryValues,
    started_up: bool,
}

struct Watchers {
    submenus_open: Watcher<i32>,
    current_section_frames: Watcher<i32>,
    acumm_frames: Watcher<i32>,
}

struct MemoryValues {
    submenus_open: Pair<i32>,
    current_section_frames: Pair<i32>,
    acumm_frames: Pair<i32>,
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

        // todo: make this part of precess info struct and only do once
        let process = &self.process_info.as_ref().unwrap().game;

        let Ok(main_module_addr) = process.get_module_address("SOR4.exe")
        else {
            asr::print_message("COULD NOT GET MODULE ADDRESS");
            return Err("COULD NOT GET MODULE ADDRESS");
        };

        // refresh values with watcher updates
        match read_value::<i32>(process, main_module_addr, &SUBMENUS_OPEN_PATH, &mut self.watchers.submenus_open, "Submenus") {
            Ok(value) => self.values.submenus_open = value,
            Err(_) => {}
        }; 
        match read_value::<i32>(process, main_module_addr, &CURR_SECTION_FRAMES, &mut self.watchers.current_section_frames, "Current section frames") {
            Ok(value) => self.values.current_section_frames = value,
            Err(_) => {}
        };
        match read_value::<i32>(process, main_module_addr, &TOTAL_FRAME_COUNT, &mut self.watchers.acumm_frames, "Accumulated Frames") {
            Ok(value) => self.values.acumm_frames = value,
            Err(_) => {}
        }

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

        // refresh mem values if possible, the values will be updated using the watchers
        if self.refresh_mem_values().is_err() {
            return;
        }

        // reset condition
        if self.values.submenus_open.current == 0 && self.values.submenus_open.old == 2 {
            asr::timer::reset();
        }

        // igt
        // todo: make igt into a atruct with values to add in a Pair and implement methods to only update it if the new calculated value is appropriate
        //       and make it part of state
        
        if self.values.current_section_frames.current - 1000 < self.values.current_section_frames.old {
            let igt_value = (self.values.current_section_frames.current + self.values.acumm_frames.current) as i64 * 1000 / 60;
            let game_time = asr::time::Duration::milliseconds(igt_value);
            asr::timer::set_game_time(game_time);
        }


    }
}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    process_info: None,
    watchers: Watchers {
        submenus_open: Watcher::new(),
        current_section_frames: Watcher::new(),
        acumm_frames: Watcher::new(),
    },
    values: MemoryValues { 
        submenus_open: Pair { old: 0, current: 0 }, 
        current_section_frames: Pair { old: 0, current: 0 },
        acumm_frames: Pair { old: 0, current: 0 }
    },
    started_up: false,
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
