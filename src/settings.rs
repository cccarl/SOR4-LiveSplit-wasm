#[derive(asr::Settings)]
pub struct Settings {
    #[default = true]
    /// Boss Rush - Boss Defeated
    pub splits_boss_rush_new_boss: bool,
    #[default = true]
    /// Boss Rush Completed
    pub splits_boss_rush_complete: bool,
    #[default = false]
    /// Survival Mode - Level Complete
    pub splits_survival: bool,
    #[default = false]
    /// Streets
    pub splits_stage1_1: bool,
    #[default = false]
    /// Sewers
    pub splits_stage1_2: bool,
    #[default = true]
    /// Diva
    pub splits_stage1_3: bool,
    #[default = false]
    /// Jail
    pub splits_stage2_1: bool,
    #[default = false]
    /// HQ
    pub splits_stage2_2: bool,
    #[default = true]
    /// Commissioner
    pub splits_stage2_3: bool,
    #[default = false]
    /// Outside
    pub splits_stage3_1a: bool,
    #[default = false]
    /// Inside
    pub splits_stage3_1b: bool,
    #[default = false]
    /// Hallway
    pub splits_stage3_1c: bool,
    #[default = true]
    /// Nora
    pub splits_stage3_2: bool,
    #[default = false]
    /// Pier
    pub splits_stage4_1: bool,
    #[default = false]
    /// Estel Start
    pub splits_music_level04_boss: bool,
    #[default = true]
    /// Estel
    pub splits_stage4_2: bool,
    #[default = false]
    /// Underground
    pub splits_stage5_1: bool,
    #[default = false]
    /// Bar
    pub splits_stage5_2: bool,
    #[default = true]
    /// Barbon
    pub splits_stage5_3: bool,
    #[default = false]
    /// Streets
    pub splits_stage6_1: bool,
    #[default = false]
    /// Dojo - Galsia Room
    pub splits_stage6_2a: bool,
    #[default = false]
    /// Dojo - Donovan Room
    pub splits_stage6_2b: bool,
    #[default = false]
    /// Dojo - Pheasant Room
    pub splits_stage6_2c: bool,
    #[default = true]
    /// Shiva
    pub splits_stage6_3: bool,
    #[default = false]
    /// Estel Start
    pub splits_music_level07_boss: bool,
    #[default = true]
    /// Estel
    pub splits_stage7_1: bool,
    #[default = false]
    /// Gallery
    pub splits_stage8_1: bool,
    #[default = true]
    /// Beyo and Riha
    pub splits_stage8_2: bool,
    #[default = false]
    /// Sauna
    pub splits_stage9_1: bool,
    #[default = false]
    /// Elevator
    pub splits_stage9_2: bool,
    #[default = true]
    /// Max
    pub splits_stage9_3: bool,
    #[default = false]
    /// Rooftops - Arrival
    pub splits_stage10_1a: bool,
    #[default = false]
    /// Rooftops - Advance
    pub splits_stage10_1b: bool,
    #[default = false]
    /// Rooftops - Wrecking Balls
    pub splits_stage10_1c: bool,
    #[default = true]
    /// DJ K-Washi
    pub splits_stage10_3: bool,
    #[default = false]
    /// Platform
    pub splits_stage11_1: bool,
    #[default = false]
    /// Boarding the Airplane
    pub splits_stage11_2a: bool,
    #[default = false]
    /// Inside the Airplane
    pub splits_stage11_2b: bool,
    #[default = true]
    /// Mr. Y
    pub splits_stage11_3: bool,
    #[default = false]
    /// Wreckage
    pub splits_stage12_1: bool,
    #[default = false]
    /// Hallway
    pub splits_stage12_2a: bool,
    #[default = false]
    /// Inside Castle
    pub splits_stage12_2b: bool,
    #[default = false]
    /// Ms. Y
    pub splits_stage12_2c: bool,
    #[default = true]
    /// Ms. Y, Mr. Y and Y Mecha
    pub splits_stage12_3: bool,
}
