use spinning_top::{const_spinlock, Spinlock};
use asr::{Process, watcher::{Watcher, Pair}};

const SUBMENUS_OPEN_PATH: [u64; 4] = [0x014B_FAB0, 0x0, 0x78, 0x28];

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

impl State {

    fn startup(&mut self) {
        asr::print_message("Started up!!!!!");
        self.started_up = true;
    }

    fn init(&mut self) {
        asr::print_message("ATTACHED!!!!!!!!!!");
        asr::set_tick_rate(60.0);
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
        let Ok(main_module_addr) = process.get_module_address("SOR4.exe")
        else {
            asr::print_message("COULD NOT GET MODULE ADDRESS");
            return;
        };

        let submenus_open = process.read_pointer_path64::<i32>(main_module_addr.0, &SUBMENUS_OPEN_PATH).unwrap();
        asr::print_message(&format!("Acum frames: {}", submenus_open) );
        
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
