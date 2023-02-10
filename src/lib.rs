use spinning_top::{const_spinlock, Spinlock};
use asr::{Process, watcher::{Watcher, Pair}};
use bytemuck;

const SUBMENUS_OPEN_PATH: [u64; 4] = [0x014B_FAB0, 0x0, 0x78, 0x28];
const CURR_SECTION_FRAMES: [u64; 4] = [0x014BFE38, 0x10, 0xA8, 0x38];
const TOTAL_FRAME_COUNT: [u64; 5] = [0x014BFE38, 0x0, 0x78, 0x10, 0x2C];

fn read_value<T: Copy + std::fmt::Display + bytemuck::Pod>(process: &Process, main_module_addr: asr::Address, pointer_path: &[u64], watcher: &mut Watcher<T>, var_key: &str) -> Pair<T> {

    match process.read_pointer_path64::<T>(main_module_addr.0, pointer_path) {
        Ok(mem_value) => {
            asr::timer::set_variable(var_key, &format!("{}", mem_value));
            //asr::print_message(&format!("{}", mem_value));
            watcher.update(Some(mem_value)).copied().unwrap()
        },
        Err(e) => {
            asr::print_message(&format!("Could not refresh Submenus open value: {:?}", e));
            watcher.update(None).copied().unwrap()
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

impl Watchers {

    fn refresh(&mut self, process: &Process) -> Result<MemoryValues, &str> {

        let Ok(main_module_addr) = process.get_module_address("SOR4.exe")
        else {
            asr::print_message("COULD NOT GET MODULE ADDRESS");
            return Err("COULD NOT GET MODULE ADDRESS");
        };

        // refresh watchers
        let return_vals = MemoryValues {
            submenus_open: read_value::<i32>(process, main_module_addr, &SUBMENUS_OPEN_PATH, &mut self.submenus_open, "Submenus"),                
            current_section_frames: read_value::<i32>(process, main_module_addr, &CURR_SECTION_FRAMES, &mut self.current_section_frames, "Current section frames"),
            acumm_frames: read_value::<i32>(process, main_module_addr, &TOTAL_FRAME_COUNT, &mut self.acumm_frames, "Accumulated Frames"),
        };

        Ok(return_vals)

    }

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
        asr::timer::set_variable("IGT", "-");
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

        let process = &self.process_info.as_ref().unwrap().game;
        // directly access the mem values using a struct made of Pairs, watchers only return a mem value after using update() so this abstracts that
        let Ok(mem_values) = self.watchers.refresh(process) else { return };

        // reset condition
        if mem_values.submenus_open.current == 0 && mem_values.submenus_open.old == 2 {
            asr::timer::reset();
        }

        // igt
        let game_time = asr::time::Duration::seconds((mem_values.current_section_frames.current + mem_values.acumm_frames.current / 60).try_into().unwrap()  );
        asr::timer::set_game_time(game_time);


    }
}

static LS_CONTROLLER: Spinlock<State> = const_spinlock(State {
    process_info: None,
    watchers: Watchers {
        submenus_open: Watcher::new(),
        current_section_frames: Watcher::new(),
        acumm_frames: Watcher::new(),
    },
    started_up: false,
});

#[no_mangle]
pub extern "C" fn update() {
    LS_CONTROLLER.lock().update();
}
