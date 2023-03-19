use crate::{State, settings::Settings};

fn get_setting_from_string(key: &str, settings: &Settings) -> bool {

    match key {
        "stage1_1" => settings.splits_stage1_1,
        "stage1_2" => settings.splits_stage1_2,
        "stage1_3" => settings.splits_stage1_3,
        "stage2_1" => settings.splits_stage2_1,
        "stage2_2" => settings.splits_stage2_2,
        "stage2_3" => settings.splits_stage2_3,
        "stage3_1a" => settings.splits_stage3_1a,
        "stage3_1b" => settings.splits_stage3_1b,
        "stage3_1c" => settings.splits_stage3_1c,
        "stage3_2" => settings.splits_stage3_2,
        "stage4_1" => settings.splits_stage4_1,
        "Music_Level04!BOSS" => settings.splits_music_level04_boss,
        "stage4_2" => settings.splits_stage4_2,
        "stage5_1" => settings.splits_stage5_1,
        "stage5_2" => settings.splits_stage5_2,
        "stage5_3" => settings.splits_stage5_3,
        "stage6_1" => settings.splits_stage6_1,
        "stage6_2a" => settings.splits_stage6_2a,
        "stage6_2b" => settings.splits_stage6_2b,
        "stage6_2c" => settings.splits_stage6_2c,
        "stage6_3" => settings.splits_stage6_3,
        "Music_Level07!BOSS" => settings.splits_music_level07_boss,
        "stage7_1" => settings.splits_stage7_1,
        "stage8_1" => settings.splits_stage8_1,
        "stage8_2" => settings.splits_stage8_2,
        "stage9_1" => settings.splits_stage9_1,
        "stage9_2" => settings.splits_stage9_2,
        "stage9_3" => settings.splits_stage9_3,
        "stage10_1a" => settings.splits_stage10_1a,
        "stage10_1b" => settings.splits_stage10_1b,
        "stage10_1c" => settings.splits_stage10_1c,
        "stage10_3" => settings.splits_stage10_3,
        "stage11_1" => settings.splits_stage11_1,
        "stage11_2a" => settings.splits_stage11_2a,
        "stage11_2b" => settings.splits_stage11_2b,
        "stage11_3" => settings.splits_stage11_3,
        "stage12_1" => settings.splits_stage12_1,
        "stage12_2a" => settings.splits_stage12_2a,
        "stage12_2b" => settings.splits_stage12_2b,
        "stage12_2c" => settings.splits_stage12_2c,
        "stage12_3" => settings.splits_stage12_3,
        "splits_llenge_01_bossrun_v3" => settings.splits_boss_rush_complete,

        _ => false,
    }
}

impl State {
    pub fn should_split(&mut self) -> bool {

        let Some(settings) = &self.settings else { return false };

        let mut split = false;

        // level/section change
        if self.values.accum_frames.current > self.values.accum_frames.old && get_setting_from_string(&self.values.current_lvl.old, settings) {
            split = true;
        } 
        // music change
        if self.values.current_music.changed() && get_setting_from_string(&self.values.current_music.current, settings) {
            split = true;
        }
        // boss rush music splits
        if self.values.current_music.changed() && self.values.current_music.current.contains("BossRush") && self.values.current_music.current != "Music_BossRush!A00_Diva" && settings.splits_boss_rush_new_boss {
            split = true;
        }

        // survival
        if self.values.accum_frames_survival.increased() && settings.splits_survival {
            asr::print_message(&format!("{} -> {}", self.values.accum_frames_survival.old, self.values.accum_frames_survival.current));
            split = true;
        }

        split
        
    }

}
