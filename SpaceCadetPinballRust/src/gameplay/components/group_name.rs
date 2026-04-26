pub const BACKGROUND_GROUP_NAME: &str = "background";
pub const TABLE_GROUP_NAME: &str = "table";
pub const BALL_GROUP_NAME: &str = "ball";
pub const PLUNGER_GROUP_NAME: &str = "plunger";
pub const LEFT_FLIPPER_GROUP_NAME: &str = "a_flip1";
pub const RIGHT_FLIPPER_GROUP_NAME: &str = "a_flip2";
pub const FONT_GROUP_NAME: &str = "font1";
pub const SCORE_GROUP_NAME: &str = "score1";
pub const BALLCOUNT_GROUP_NAME: &str = "ballcount1";
pub const PLAYER_NUMBER_GROUP_NAME: &str = "player_number1";
pub const INFO_TEXT_BOX_GROUP_NAME: &str = "info_text_box";
pub const MISSION_TEXT_BOX_GROUP_NAME: &str = "mission_text_box";
pub const BALL_READY_LIGHT_GROUP_NAME: &str = "lite1";
pub const LEFT_FLIPPER_LIGHT_GROUP_NAME: &str = "lite2";
pub const RIGHT_FLIPPER_LIGHT_GROUP_NAME: &str = "lite3";
pub const BALL_IN_PLAY_LIGHT_GROUP_NAME: &str = "lite84";
pub const BALL_DRAINED_LIGHT_GROUP_NAME: &str = "lite85";
pub const PLUNGER_CHARGE_LIGHT_GROUPS: [(&str, f32); 8] = [
    ("lite39", 0.15),
    ("lite40", 0.30),
    ("lite44", 0.45),
    ("lite45", 0.55),
    ("lite46", 0.65),
    ("lite47", 0.75),
    ("lite48", 0.85),
    ("lite49", 0.95),
];
pub const SCORE_MILESTONE_LIGHT_GROUPS: [(&str, u64); 8] = [
    ("lite101", 1000),
    ("lite102", 2000),
    ("lite107", 3000),
    ("lite108", 4000),
    ("lite130", 5000),
    ("lite131", 6000),
    ("lite132", 7000),
    ("lite133", 8000),
];
pub const LAUNCH_MILESTONE_LIGHT_GROUPS: [(&str, u64); 6] = [
    ("lite103", 1),
    ("lite104", 2),
    ("lite109", 3),
    ("lite154", 4),
    ("lite155", 5),
    ("lite156", 6),
];
pub const DRAIN_MILESTONE_LIGHT_GROUPS: [(&str, u64); 6] = [
    ("lite105", 1),
    ("lite106", 2),
    ("lite110", 3),
    ("lite157", 4),
    ("lite158", 5),
    ("lite159", 6),
];
pub const DEFAULT_TABLE_LIGHT_GROUPS: [&str; 88] = [
    "lite4", "lite5", "lite6", "lite7", "lite8", "lite9", "lite10", "lite11", "lite12", "lite13",
    "lite16", "lite17", "lite18", "lite19", "lite20", "lite21", "lite22", "lite23", "lite24",
    "lite25", "lite26", "lite27", "lite28", "lite29", "lite30", "lite38", "lite50", "lite51",
    "lite52", "lite54", "lite55", "lite56", "lite58", "lite59", "lite60", "lite61", "lite62",
    "lite63", "lite64", "lite65", "lite66", "lite67", "lite68", "lite69", "lite70", "lite71",
    "lite72", "lite77", "lite144", "lite145", "lite146", "lite147", "lite148", "lite149",
    "lite150", "lite151", "lite152", "lite160", "lite161", "lite162", "lite169", "lite170",
    "lite171", "lite185", "lite186", "lite187", "lite188", "lite189", "lite190", "lite191",
    "lite192", "lite193", "lite194", "lite195", "lite196", "lite198", "lite199", "lite200",
    "lite300", "lite301", "lite302", "lite303", "lite304", "lite305", "lite306", "lite307",
    "lite308", "lite309",
];
pub const DEFAULT_ROLLOVER_LIGHT_GROUPS: [&str; 14] = [
    "lite310",
    "lite311",
    "lite312",
    "lite313",
    "lite314",
    "lite315",
    "lite316",
    "lite317",
    "lite318",
    "lite319",
    "lite320",
    "lite321",
    "lite322",
    "literoll179",
];
pub const DEFAULT_FUEL_ROLLOVER_LIGHT_GROUPS: [&str; 5] = [
    "literoll180",
    "literoll181",
    "literoll182",
    "literoll183",
    "literoll184",
];
pub const BUMPER_SEQUENCE_GROUPS: [&str; 7] = [
    "a_bump1", "a_bump2", "a_bump3", "a_bump4", "a_bump5", "a_bump6", "a_bump7",
];
pub const FLAG_SEQUENCE_GROUPS: [&str; 2] = ["a_flag1", "a_flag2"];
pub const GATE_SEQUENCE_GROUPS: [&str; 2] = ["v_gate1", "v_gate2"];
pub const KICKBACK_SEQUENCE_GROUPS: [&str; 2] = ["a_kick1", "a_kick2"];
pub const KICKOUT_SEQUENCE_GROUPS: [&str; 3] = ["a_kout1", "a_kout2", "a_kout3"];
pub const SINK_SEQUENCE_GROUPS: [&str; 4] = ["v_sink1", "v_sink2", "v_sink3", "v_sink7"];
pub const ONEWAY_SEQUENCE_GROUPS: [&str; 9] = [
    "s_onewy1",
    "s_onewy4",
    "s_onewy7",
    "s_onewy8",
    "s_onewy9",
    "s_onewy10",
    "s_onewy11",
    "s_onewy12",
    "s_onewy13",
];
pub const REBOUNDER_SEQUENCE_GROUPS: [&str; 4] = ["v_rebo1", "v_rebo2", "v_rebo3", "v_rebo4"];
pub const ROLLOVER_SEQUENCE_GROUPS: [&str; 18] = [
    "a_roll1",
    "a_roll2",
    "a_roll3",
    "a_roll4",
    "a_roll5",
    "a_roll6",
    "a_roll7",
    "a_roll8",
    "a_roll9",
    "a_roll110",
    "a_roll111",
    "a_roll112",
    "a_roll179",
    "a_roll180",
    "a_roll181",
    "a_roll182",
    "a_roll183",
    "a_roll184",
];
pub const STATIC_TABLE_SEQUENCE_GROUPS: [&str; 3] = ["v_bloc1", "ramp", "ramp_hole"];
pub const FUEL_BARGRAPH_GROUP_NAME: &str = "fuel_bargraph";
pub const MIDDLE_CIRCLE_GROUP_NAME: &str = "middle_circle";
pub const OUTER_CIRCLE_GROUP_NAME: &str = "outer_circle";
pub const GOAL_LIGHTS_GROUP_NAME: &str = "goal_lights";
pub const HYPERSPACE_LIGHTS_GROUP_NAME: &str = "hyperspace_lights";
pub const SKILL_SHOT_LIGHTS_GROUP_NAME: &str = "skill_shot_lights";
pub const WORM_HOLE_LIGHTS_GROUP_NAME: &str = "worm_hole_lights";
pub const LIGHT_GROUP_SEQUENCE_GROUPS: [&str; 12] = [
    "lchute_tgt_lights",
    "l_trek_lights",
    "right_target_lights",
    "r_trek_lights",
    "bmpr_inc_lights",
    "bpr_solotgt_lights",
    "bsink_arrow_lights",
    "bumper_target_lights",
    "ramp_bmpr_inc_lights",
    "ramp_tgt_lights",
    "top_circle_tgt_lights",
    "top_target_lights",
];
pub const TARGET_SEQUENCE_GROUPS: [&str; 22] = [
    "a_targ1", "a_targ2", "a_targ3", "a_targ4", "a_targ5", "a_targ6", "a_targ7", "a_targ8",
    "a_targ9", "a_targ10", "a_targ11", "a_targ12", "a_targ13", "a_targ14", "a_targ15", "a_targ16",
    "a_targ17", "a_targ18", "a_targ19", "a_targ20", "a_targ21", "a_targ22",
];
pub const TRIPWIRE_SEQUENCE_GROUPS: [&str; 5] =
    ["s_trip1", "s_trip2", "s_trip3", "s_trip4", "s_trip5"];
