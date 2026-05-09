use crate::assets::DatFile;
use crate::gameplay::components::group_name::{
    FUEL_BARGRAPH_GROUP_NAME, LEFT_FLIPPER_GROUP_NAME, PLUNGER_GROUP_NAME,
};
use crate::gameplay::components::{ComponentId, ComponentState, GameplayComponent};
use crate::gameplay::mechanics::{
    BlockerMechanic, DrainMechanic, FlipperMechanic, GateMechanic, KickbackMechanic,
    KickoutMechanic, LightBargraphMechanic, LightGroupMechanic, LightMechanic,
    LightRolloverMechanic, OnewayMechanic, PlungerMechanic, PopupTargetMechanic,
    RolloverMechanic, SinkMechanic, SoloTargetMechanic, TripwireMechanic, WallMechanic,
};

use super::PinballTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentKind {
    Flipper,
    Plunger,
    Drain,
    Blocker,
    Gate,
    Kickback,
    Kickout,
    Sink,
    Light,
    LightBargraph,
    LightGroup,
    PopupTarget,
    SoloTarget,
    Wall,
    Oneway,
    Rollover,
    LightRollover,
    Tripwire,
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub id: ComponentId,
    pub name: &'static str,
    pub kind: ComponentKind,
    pub source_class: &'static str,
    pub control_name: Option<&'static str>,
    pub primary_group_name: Option<&'static str>,
    pub scoring: &'static [i32],
}

impl ComponentDefinition {
    pub const fn new(
        id: ComponentId,
        name: &'static str,
        kind: ComponentKind,
        source_class: &'static str,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            source_class,
            control_name: None,
            primary_group_name: None,
            scoring: &[],
        }
    }

    pub const fn with_control(mut self, control_name: &'static str) -> Self {
        self.control_name = Some(control_name);
        self
    }

    pub const fn with_primary_group(mut self, group_name: &'static str) -> Self {
        self.primary_group_name = Some(group_name);
        self
    }

    pub const fn with_scoring(mut self, scoring: &'static [i32]) -> Self {
        self.scoring = scoring;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableLinkReport {
    pub component_count: usize,
    pub resolved_group_count: usize,
    pub missing_groups: Vec<&'static str>,
}

pub fn default_component_definitions() -> Vec<ComponentDefinition> {
    let mut definitions = vec![
        ComponentDefinition::new(
            ComponentId(1),
            "flipper",
            ComponentKind::Flipper,
            "TFlipper",
        )
        .with_control("FlipperControl")
        .with_primary_group(LEFT_FLIPPER_GROUP_NAME),
        ComponentDefinition::new(
            ComponentId(2),
            "plunger",
            ComponentKind::Plunger,
            "TPlunger",
        )
        .with_control("PlungerControl")
        .with_primary_group(PLUNGER_GROUP_NAME),
        ComponentDefinition::new(ComponentId(3), "drain", ComponentKind::Drain, "TDrain")
            .with_control("BallDrainControl"),
    ];

    definitions.extend([
        ComponentDefinition::new(
            ComponentId(4),
            "v_bloc1",
            ComponentKind::Blocker,
            "TBlocker",
        )
        .with_control("DrainBallBlockerControl")
        .with_primary_group("v_bloc1"),
        ComponentDefinition::new(ComponentId(5), "v_gate1", ComponentKind::Gate, "TGate")
            .with_control("LeftKickerGateControl")
            .with_primary_group("v_gate1"),
        ComponentDefinition::new(ComponentId(6), "v_gate2", ComponentKind::Gate, "TGate")
            .with_control("RightKickerGateControl")
            .with_primary_group("v_gate2"),
        ComponentDefinition::new(
            ComponentId(7),
            "a_kick1",
            ComponentKind::Kickback,
            "TKickback",
        )
        .with_control("LeftKickerControl")
        .with_primary_group("a_kick1"),
        ComponentDefinition::new(
            ComponentId(8),
            "a_kick2",
            ComponentKind::Kickback,
            "TKickback",
        )
        .with_control("RightKickerControl")
        .with_primary_group("a_kick2"),
        ComponentDefinition::new(
            ComponentId(9),
            "a_kout1",
            ComponentKind::Kickout,
            "TKickout",
        )
        .with_control("GravityWellKickoutControl")
        .with_primary_group("a_kout1"),
        ComponentDefinition::new(
            ComponentId(10),
            "a_kout2",
            ComponentKind::Kickout,
            "TKickout",
        )
        .with_control("HyperspaceKickOutControl")
        .with_primary_group("a_kout2"),
        ComponentDefinition::new(
            ComponentId(11),
            "a_kout3",
            ComponentKind::Kickout,
            "TKickout",
        )
        .with_control("BlackHoleKickoutControl")
        .with_primary_group("a_kout3"),
        ComponentDefinition::new(ComponentId(12), "v_sink1", ComponentKind::Sink, "TSink")
            .with_control("WormHoleControl")
            .with_primary_group("v_sink1"),
        ComponentDefinition::new(ComponentId(13), "v_sink2", ComponentKind::Sink, "TSink")
            .with_control("WormHoleControl")
            .with_primary_group("v_sink2"),
        ComponentDefinition::new(ComponentId(14), "v_sink3", ComponentKind::Sink, "TSink")
            .with_control("WormHoleControl")
            .with_primary_group("v_sink3"),
        ComponentDefinition::new(ComponentId(15), "v_sink7", ComponentKind::Sink, "TSink")
            .with_control("EscapeChuteSinkControl")
            .with_primary_group("v_sink7"),
    ]);

    definitions.extend(sensor_component_definitions());
    definitions.extend(wall_component_definitions());
    definitions.extend(solo_target_component_definitions());
    definitions.extend(popup_target_component_definitions());
    definitions.extend(light_component_definitions());
    definitions.extend(light_bargraph_component_definitions());
    definitions.extend(light_group_component_definitions());

    definitions
}

fn sensor_component_definitions() -> [ComponentDefinition; 26] {
    [
        ComponentDefinition::new(
            ComponentId(16),
            "s_onewy1",
            ComponentKind::Oneway,
            "TOneway",
        )
        .with_control("SkillShotGate1Control")
        .with_primary_group("s_onewy1"),
        ComponentDefinition::new(
            ComponentId(17),
            "s_onewy4",
            ComponentKind::Oneway,
            "TOneway",
        )
        .with_control("DeploymentChuteToEscapeChuteOneWayControl")
        .with_primary_group("s_onewy4"),
        ComponentDefinition::new(
            ComponentId(18),
            "s_onewy10",
            ComponentKind::Oneway,
            "TOneway",
        )
        .with_control("DeploymentChuteToTableOneWayControl")
        .with_primary_group("s_onewy10"),
        ComponentDefinition::new(
            ComponentId(19),
            "a_roll1",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("ReentryLanesRolloverControl")
        .with_primary_group("a_roll1"),
        ComponentDefinition::new(
            ComponentId(20),
            "a_roll2",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("ReentryLanesRolloverControl")
        .with_primary_group("a_roll2"),
        ComponentDefinition::new(
            ComponentId(21),
            "a_roll3",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("ReentryLanesRolloverControl")
        .with_primary_group("a_roll3"),
        ComponentDefinition::new(
            ComponentId(22),
            "a_roll4",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("OutLaneRolloverControl")
        .with_primary_group("a_roll4"),
        ComponentDefinition::new(
            ComponentId(23),
            "a_roll5",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("BonusLaneRolloverControl")
        .with_primary_group("a_roll5"),
        ComponentDefinition::new(
            ComponentId(24),
            "a_roll6",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("ReturnLaneRolloverControl")
        .with_primary_group("a_roll6"),
        ComponentDefinition::new(
            ComponentId(25),
            "a_roll7",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("ReturnLaneRolloverControl")
        .with_primary_group("a_roll7"),
        ComponentDefinition::new(
            ComponentId(26),
            "a_roll8",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("OutLaneRolloverControl")
        .with_primary_group("a_roll8"),
        ComponentDefinition::new(
            ComponentId(27),
            "a_roll9",
            ComponentKind::LightRollover,
            "TLightRollover",
        )
        .with_control("SpaceWarpRolloverControl")
        .with_primary_group("a_roll9"),
        ComponentDefinition::new(
            ComponentId(28),
            "a_roll110",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("LaunchLanesRolloverControl")
        .with_primary_group("a_roll110"),
        ComponentDefinition::new(
            ComponentId(29),
            "a_roll111",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("LaunchLanesRolloverControl")
        .with_primary_group("a_roll111"),
        ComponentDefinition::new(
            ComponentId(30),
            "a_roll112",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("LaunchLanesRolloverControl")
        .with_primary_group("a_roll112"),
        ComponentDefinition::new(
            ComponentId(31),
            "a_roll179",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover1Control")
        .with_primary_group("a_roll179")
        .with_scoring(&[500]),
        ComponentDefinition::new(
            ComponentId(32),
            "a_roll180",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover2Control")
        .with_primary_group("a_roll180")
        .with_scoring(&[500]),
        ComponentDefinition::new(
            ComponentId(33),
            "a_roll181",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover3Control")
        .with_primary_group("a_roll181")
        .with_scoring(&[500]),
        ComponentDefinition::new(
            ComponentId(34),
            "a_roll182",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover4Control")
        .with_primary_group("a_roll182")
        .with_scoring(&[500]),
        ComponentDefinition::new(
            ComponentId(35),
            "a_roll183",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover5Control")
        .with_primary_group("a_roll183")
        .with_scoring(&[500]),
        ComponentDefinition::new(
            ComponentId(36),
            "a_roll184",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover6Control")
        .with_primary_group("a_roll184")
        .with_scoring(&[500]),
        ComponentDefinition::new(
            ComponentId(37),
            "s_trip1",
            ComponentKind::Tripwire,
            "TTripwire",
        )
        .with_control("SkillShotGate2Control")
        .with_primary_group("s_trip1"),
        ComponentDefinition::new(
            ComponentId(38),
            "s_trip2",
            ComponentKind::Tripwire,
            "TTripwire",
        )
        .with_control("SkillShotGate3Control")
        .with_primary_group("s_trip2"),
        ComponentDefinition::new(
            ComponentId(39),
            "s_trip3",
            ComponentKind::Tripwire,
            "TTripwire",
        )
        .with_control("SkillShotGate4Control")
        .with_primary_group("s_trip3"),
        ComponentDefinition::new(
            ComponentId(40),
            "s_trip4",
            ComponentKind::Tripwire,
            "TTripwire",
        )
        .with_control("SkillShotGate5Control")
        .with_primary_group("s_trip4"),
        ComponentDefinition::new(
            ComponentId(41),
            "s_trip5",
            ComponentKind::Tripwire,
            "TTripwire",
        )
        .with_control("SkillShotGate6Control")
        .with_primary_group("s_trip5"),
    ]
}

fn wall_component_definitions() -> [ComponentDefinition; 4] {
    [
        ComponentDefinition::new(ComponentId(42), "v_rebo1", ComponentKind::Wall, "TWall")
            .with_control("FlipperRebounderControl1")
            .with_primary_group("v_rebo1"),
        ComponentDefinition::new(ComponentId(43), "v_rebo2", ComponentKind::Wall, "TWall")
            .with_control("FlipperRebounderControl2")
            .with_primary_group("v_rebo2"),
        ComponentDefinition::new(ComponentId(44), "v_rebo3", ComponentKind::Wall, "TWall")
            .with_control("RebounderControl")
            .with_primary_group("v_rebo3"),
        ComponentDefinition::new(ComponentId(45), "v_rebo4", ComponentKind::Wall, "TWall")
            .with_control("RebounderControl")
            .with_primary_group("v_rebo4"),
    ]
}

fn solo_target_component_definitions() -> [ComponentDefinition; 13] {
    [
        ComponentDefinition::new(ComponentId(46), "a_targ10", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("FuelSpotTargetControl")
            .with_primary_group("a_targ10")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(47), "a_targ11", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("FuelSpotTargetControl")
            .with_primary_group("a_targ11")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(48), "a_targ12", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("FuelSpotTargetControl")
            .with_primary_group("a_targ12")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(49), "a_targ13", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("MissionSpotTargetControl")
            .with_primary_group("a_targ13")
            .with_scoring(&[1000]),
        ComponentDefinition::new(ComponentId(50), "a_targ14", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("MissionSpotTargetControl")
            .with_primary_group("a_targ14")
            .with_scoring(&[1000]),
        ComponentDefinition::new(ComponentId(51), "a_targ15", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("MissionSpotTargetControl")
            .with_primary_group("a_targ15")
            .with_scoring(&[1000]),
        ComponentDefinition::new(ComponentId(52), "a_targ16", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("LeftHazardSpotTargetControl")
            .with_primary_group("a_targ16")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(53), "a_targ17", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("LeftHazardSpotTargetControl")
            .with_primary_group("a_targ17")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(54), "a_targ18", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("LeftHazardSpotTargetControl")
            .with_primary_group("a_targ18")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(55), "a_targ19", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("RightHazardSpotTargetControl")
            .with_primary_group("a_targ19")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(56), "a_targ20", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("RightHazardSpotTargetControl")
            .with_primary_group("a_targ20")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(57), "a_targ21", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("RightHazardSpotTargetControl")
            .with_primary_group("a_targ21")
            .with_scoring(&[750]),
        ComponentDefinition::new(ComponentId(58), "a_targ22", ComponentKind::SoloTarget, "TSoloTarget")
            .with_control("WormHoleDestinationControl")
            .with_primary_group("a_targ22")
            .with_scoring(&[750]),
    ]
}

fn popup_target_component_definitions() -> [ComponentDefinition; 9] {
    [
        ComponentDefinition::new(ComponentId(59), "a_targ1", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("BoosterTargetControl")
            .with_primary_group("a_targ1")
            .with_scoring(&[500, 5000]),
        ComponentDefinition::new(ComponentId(60), "a_targ2", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("BoosterTargetControl")
            .with_primary_group("a_targ2")
            .with_scoring(&[500, 5000]),
        ComponentDefinition::new(ComponentId(61), "a_targ3", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("BoosterTargetControl")
            .with_primary_group("a_targ3")
            .with_scoring(&[500, 5000]),
        ComponentDefinition::new(ComponentId(62), "a_targ4", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("MedalTargetControl")
            .with_primary_group("a_targ4")
            .with_scoring(&[1500, 10000, 50000]),
        ComponentDefinition::new(ComponentId(63), "a_targ5", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("MedalTargetControl")
            .with_primary_group("a_targ5")
            .with_scoring(&[1500, 10000, 50000]),
        ComponentDefinition::new(ComponentId(64), "a_targ6", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("MedalTargetControl")
            .with_primary_group("a_targ6")
            .with_scoring(&[1500, 10000, 50000]),
        ComponentDefinition::new(ComponentId(65), "a_targ7", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("MultiplierTargetControl")
            .with_primary_group("a_targ7")
            .with_scoring(&[500, 1500]),
        ComponentDefinition::new(ComponentId(66), "a_targ8", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("MultiplierTargetControl")
            .with_primary_group("a_targ8")
            .with_scoring(&[500, 1500]),
        ComponentDefinition::new(ComponentId(67), "a_targ9", ComponentKind::PopupTarget, "TPopupTarget")
            .with_control("MultiplierTargetControl")
            .with_primary_group("a_targ9")
            .with_scoring(&[500, 1500]),
    ]
}

fn light_component_definitions() -> [ComponentDefinition; 22] {
    [
        ComponentDefinition::new(ComponentId(68), "lite58", ComponentKind::Light, "TLight")
            .with_control("BonusHoldLightControl")
            .with_primary_group("lite58"),
        ComponentDefinition::new(ComponentId(69), "lite59", ComponentKind::Light, "TLight")
            .with_control("BonusLightControl")
            .with_primary_group("lite59"),
        ComponentDefinition::new(ComponentId(70), "lite60", ComponentKind::Light, "TLight")
            .with_control("JackpotLightControl")
            .with_primary_group("lite60"),
        ComponentDefinition::new(ComponentId(71), "lite61", ComponentKind::Light, "TLight")
            .with_control("FlagLightControl")
            .with_primary_group("lite61"),
        ComponentDefinition::new(ComponentId(72), "literoll179", ComponentKind::Light, "TLight")
            .with_primary_group("literoll179"),
        ComponentDefinition::new(ComponentId(73), "literoll180", ComponentKind::Light, "TLight")
            .with_primary_group("literoll180"),
        ComponentDefinition::new(ComponentId(74), "literoll181", ComponentKind::Light, "TLight")
            .with_primary_group("literoll181"),
        ComponentDefinition::new(ComponentId(75), "literoll182", ComponentKind::Light, "TLight")
            .with_primary_group("literoll182"),
        ComponentDefinition::new(ComponentId(76), "literoll183", ComponentKind::Light, "TLight")
            .with_primary_group("literoll183"),
        ComponentDefinition::new(ComponentId(77), "literoll184", ComponentKind::Light, "TLight")
            .with_primary_group("literoll184"),
        ComponentDefinition::new(ComponentId(78), "lite70", ComponentKind::Light, "TLight")
            .with_primary_group("lite70"),
        ComponentDefinition::new(ComponentId(79), "lite71", ComponentKind::Light, "TLight")
            .with_primary_group("lite71"),
        ComponentDefinition::new(ComponentId(80), "lite72", ComponentKind::Light, "TLight")
            .with_primary_group("lite72"),
        ComponentDefinition::new(ComponentId(81), "lite101", ComponentKind::Light, "TLight")
            .with_primary_group("lite101"),
        ComponentDefinition::new(ComponentId(82), "lite102", ComponentKind::Light, "TLight")
            .with_primary_group("lite102"),
        ComponentDefinition::new(ComponentId(83), "lite103", ComponentKind::Light, "TLight")
            .with_primary_group("lite103"),
        ComponentDefinition::new(ComponentId(84), "lite104", ComponentKind::Light, "TLight")
            .with_primary_group("lite104"),
        ComponentDefinition::new(ComponentId(85), "lite105", ComponentKind::Light, "TLight")
            .with_primary_group("lite105"),
        ComponentDefinition::new(ComponentId(86), "lite106", ComponentKind::Light, "TLight")
            .with_primary_group("lite106"),
        ComponentDefinition::new(ComponentId(87), "lite107", ComponentKind::Light, "TLight")
            .with_primary_group("lite107"),
        ComponentDefinition::new(ComponentId(88), "lite108", ComponentKind::Light, "TLight")
            .with_primary_group("lite108"),
        ComponentDefinition::new(ComponentId(89), "lite109", ComponentKind::Light, "TLight")
            .with_primary_group("lite109"),
    ]
}

fn light_group_component_definitions() -> [ComponentDefinition; 6] {
    [
        ComponentDefinition::new(
            ComponentId(91),
            "bumper_target_lights",
            ComponentKind::LightGroup,
            "TLightGroup",
        )
        .with_control("MedalLightGroupControl")
        .with_primary_group("bumper_target_lights"),
        ComponentDefinition::new(
            ComponentId(92),
            "top_target_lights",
            ComponentKind::LightGroup,
            "TLightGroup",
        )
        .with_control("MultiplierLightGroupControl")
        .with_primary_group("top_target_lights"),
        ComponentDefinition::new(
            ComponentId(93),
            "top_circle_tgt_lights",
            ComponentKind::LightGroup,
            "TLightGroup",
        )
        .with_primary_group("top_circle_tgt_lights"),
        ComponentDefinition::new(
            ComponentId(94),
            "ramp_tgt_lights",
            ComponentKind::LightGroup,
            "TLightGroup",
        )
        .with_primary_group("ramp_tgt_lights"),
        ComponentDefinition::new(
            ComponentId(95),
            "lchute_tgt_lights",
            ComponentKind::LightGroup,
            "TLightGroup",
        )
        .with_primary_group("lchute_tgt_lights"),
        ComponentDefinition::new(
            ComponentId(96),
            "bpr_solotgt_lights",
            ComponentKind::LightGroup,
            "TLightGroup",
        )
        .with_primary_group("bpr_solotgt_lights"),
    ]
}

fn light_bargraph_component_definitions() -> [ComponentDefinition; 1] {
    [ComponentDefinition::new(
        ComponentId(90),
        FUEL_BARGRAPH_GROUP_NAME,
        ComponentKind::LightBargraph,
        "TLightBargraph",
    )
    .with_primary_group(FUEL_BARGRAPH_GROUP_NAME)]
}

pub fn install_components(
    table: &mut PinballTable,
    definitions: &[ComponentDefinition],
    dat_file: Option<&DatFile>,
) -> TableLinkReport {
    let mut report = TableLinkReport {
        component_count: definitions.len(),
        ..TableLinkReport::default()
    };

    for definition in definitions {
        let group_index = definition
            .primary_group_name
            .and_then(|group_name| match dat_file {
                Some(dat_file) => match dat_file.record_labeled(group_name) {
                    Some(index) => {
                        report.resolved_group_count += 1;
                        Some(index as i32)
                    }
                    None => {
                        report.missing_groups.push(group_name);
                        None
                    }
                },
                None => None,
            });

        table.add_boxed_component(component_from_definition(definition, group_index));
    }

    report
}

fn component_from_definition(
    definition: &ComponentDefinition,
    group_index: Option<i32>,
) -> Box<dyn GameplayComponent> {
    let mut state = ComponentState::new(definition.id, definition.name);
    if let Some(control_name) = definition.control_name {
        state = state.with_control(control_name);
    }
    if let Some(group_index) = group_index {
        state = state.with_group_index(group_index);
    }
    if !definition.scoring.is_empty() {
        state = state.with_scoring(definition.scoring.to_vec());
    }

    match definition.kind {
        ComponentKind::Flipper => Box::new(FlipperMechanic::from_state(state)),
        ComponentKind::Plunger => Box::new(PlungerMechanic::from_state(state)),
        ComponentKind::Drain => Box::new(DrainMechanic::from_state(state)),
        ComponentKind::Blocker => Box::new(BlockerMechanic::from_state(state)),
        ComponentKind::Gate => Box::new(GateMechanic::from_state(state)),
        ComponentKind::Kickback => Box::new(KickbackMechanic::from_state(state)),
        ComponentKind::Kickout => Box::new(KickoutMechanic::from_state(state)),
        ComponentKind::Sink => Box::new(SinkMechanic::from_state(state)),
        ComponentKind::Light => Box::new(LightMechanic::from_state(state)),
        ComponentKind::LightBargraph => Box::new(LightBargraphMechanic::from_state(state, 6)),
        ComponentKind::LightGroup => Box::new(LightGroupMechanic::from_state(
            state,
            if definition.name == "top_target_lights" { 4 } else { 3 },
        )),
        ComponentKind::PopupTarget => Box::new(PopupTargetMechanic::from_state(state)),
        ComponentKind::SoloTarget => Box::new(SoloTargetMechanic::from_state(state)),
        ComponentKind::Wall => Box::new(WallMechanic::from_state(state)),
        ComponentKind::Rollover => Box::new(RolloverMechanic::from_state(state)),
        ComponentKind::LightRollover => Box::new(LightRolloverMechanic::from_state(state)),
        ComponentKind::Tripwire => Box::new(TripwireMechanic::from_state(state)),
        ComponentKind::Oneway => Box::new(OnewayMechanic::from_state(state)),
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::{DatFile, EntryData, EntryPayload, FieldType, GroupData};
    use crate::gameplay::components::MessageCode;
    use crate::gameplay::TableMessage;

    use super::*;

    fn named_group(group_id: usize, group_name: &str) -> GroupData {
        GroupData {
            group_id,
            group_name: Some(group_name.to_string()),
            entries: Vec::new(),
            bitmaps: [None, None, None],
            zmaps: [None, None, None],
            needs_sort: false,
        }
    }

    fn collision_group(group_id: usize, group_name: &str) -> GroupData {
        let mut group = named_group(group_id, group_name);
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::ShortArray,
            field_size: 8,
            payload: EntryPayload::RawBytes(
                [602_i16, 2_i16, 304_i16, 17_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 24,
            payload: EntryPayload::RawBytes(
                [600.0_f32, 2.0, 10.0, 20.0, 30.0, 40.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group
    }

    fn oneway_group(group_id: usize, group_name: &str) -> GroupData {
        let mut group = named_group(group_id, group_name);
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 24,
            payload: EntryPayload::RawBytes(
                [600.0_f32, 2.0, 30.0, 40.0, 10.0, 20.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group
    }

    fn kickout_group(group_id: usize, group_name: &str) -> GroupData {
        let mut group = named_group(group_id, group_name);
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 20,
            payload: EntryPayload::RawBytes(
                [600.0_f32, 1.0, 30.0, 40.0, 5.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 8,
            payload: EntryPayload::RawBytes(
                [306.0_f32, 2.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group
    }

    fn plunger_group(group_id: usize, group_name: &str, x: f32, y: f32) -> GroupData {
        let mut group = named_group(group_id, group_name);
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [601.0_f32, x, y]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group
    }

    fn popup_target_group(group_id: usize, group_name: &str, timer_time: f32) -> GroupData {
        let mut group = named_group(group_id, group_name);
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 8,
            payload: EntryPayload::RawBytes(
                [407.0_f32, timer_time]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group
    }

    fn light_bargraph_group(group_id: usize, group_name: &str, timer_values: &[f32]) -> GroupData {
        let mut group = named_group(group_id, group_name);
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: ((timer_values.len() + 1) * std::mem::size_of::<f32>()) as i32,
            payload: EntryPayload::RawBytes(
                std::iter::once(904.0_f32)
                    .chain(timer_values.iter().copied())
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        group
    }

    #[test]
    fn default_definitions_match_current_playable_slice() {
        let definitions = default_component_definitions();

        assert_eq!(definitions.len(), 96);
        assert_eq!(definitions[0].source_class, "TFlipper");
        assert_eq!(definitions[0].control_name, Some("FlipperControl"));
        assert_eq!(definitions[1].source_class, "TPlunger");
        assert_eq!(definitions[1].primary_group_name, Some(PLUNGER_GROUP_NAME));
        assert_eq!(definitions[2].source_class, "TDrain");
        assert_eq!(definitions[2].control_name, Some("BallDrainControl"));
        assert_eq!(definitions[3].source_class, "TBlocker");
        assert_eq!(definitions[5].control_name, Some("RightKickerGateControl"));
        assert_eq!(definitions[11].primary_group_name, Some("v_sink1"));
        assert_eq!(definitions[15].source_class, "TOneway");
        assert_eq!(
            definitions[17].control_name,
            Some("DeploymentChuteToTableOneWayControl")
        );
        assert_eq!(definitions[26].source_class, "TLightRollover");
        assert_eq!(definitions[40].control_name, Some("SkillShotGate6Control"));
        assert_eq!(definitions[41].source_class, "TWall");
        assert_eq!(definitions[44].control_name, Some("RebounderControl"));
        assert_eq!(definitions[45].source_class, "TSoloTarget");
        assert_eq!(definitions[57].control_name, Some("WormHoleDestinationControl"));
        assert_eq!(definitions[58].source_class, "TPopupTarget");
        assert_eq!(definitions[66].control_name, Some("MultiplierTargetControl"));
        assert_eq!(definitions[67].source_class, "TLight");
        assert_eq!(definitions[70].control_name, Some("FlagLightControl"));
        assert_eq!(definitions[71].primary_group_name, Some("literoll179"));
        assert_eq!(definitions[76].primary_group_name, Some("literoll184"));
        assert_eq!(definitions[77].primary_group_name, Some("lite70"));
        assert_eq!(definitions[79].primary_group_name, Some("lite72"));
        assert_eq!(definitions[80].primary_group_name, Some("lite101"));
        assert_eq!(definitions[83].primary_group_name, Some("lite104"));
        assert_eq!(definitions[88].primary_group_name, Some("lite109"));
        assert_eq!(definitions[89].source_class, "TLightBargraph");
        assert_eq!(definitions[90].control_name, Some("MedalLightGroupControl"));
        assert_eq!(definitions[91].control_name, Some("MultiplierLightGroupControl"));
        assert_eq!(definitions[92].primary_group_name, Some("top_circle_tgt_lights"));
        assert_eq!(definitions[93].primary_group_name, Some("ramp_tgt_lights"));
        assert_eq!(definitions[94].primary_group_name, Some("lchute_tgt_lights"));
        assert_eq!(definitions[95].primary_group_name, Some("bpr_solotgt_lights"));
    }

    #[test]
    fn table_from_dat_resolves_primary_group_indexes() {
        let definitions = default_component_definitions();
        let groups = definitions
            .iter()
            .filter_map(|definition| definition.primary_group_name)
            .enumerate()
            .map(|(group_id, group_name)| named_group(group_id, group_name))
            .collect();
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups,
        };

        let table = PinballTable::from_dat(&dat_file);

        assert_eq!(table.component_count(), 96);
        assert_eq!(table.link_report().component_count, 96);
        assert_eq!(table.link_report().resolved_group_count, 95);
        assert!(table.link_report().missing_groups.is_empty());
        assert_eq!(
            table
                .find_component("flipper")
                .and_then(|component| component.group_index()),
            Some(0)
        );
        assert_eq!(
            table
                .find_component_by_group_index(1)
                .map(|component| component.name()),
            Some("plunger")
        );
        assert_eq!(
            table
                .find_component("v_gate2")
                .and_then(|component| component.state().control_name),
            Some("RightKickerGateControl")
        );
        assert_eq!(
            table
                .find_component("a_roll9")
                .and_then(|component| component.state().control_name),
            Some("SpaceWarpRolloverControl")
        );
        assert_eq!(
            table
                .find_component("s_trip5")
                .and_then(|component| component.state().control_name),
            Some("SkillShotGate6Control")
        );
        assert_eq!(
            table
                .find_component("v_rebo3")
                .and_then(|component| component.state().control_name),
            Some("RebounderControl")
        );
        assert_eq!(
            table
                .find_component("a_targ22")
                .and_then(|component| component.state().control_name),
            Some("WormHoleDestinationControl")
        );
        assert_eq!(
            table
                .find_component("a_targ4")
                .and_then(|component| component.state().control_name),
            Some("MedalTargetControl")
        );
        assert_eq!(
            table
                .find_component("lite60")
                .and_then(|component| component.state().control_name),
            Some("JackpotLightControl")
        );
        assert_eq!(
            table
                .find_component(FUEL_BARGRAPH_GROUP_NAME)
                .map(|component| component.name()),
            Some(FUEL_BARGRAPH_GROUP_NAME)
        );
        assert_eq!(
            table
                .find_component("top_target_lights")
                .and_then(|component| component.state().control_name),
            Some("MultiplierLightGroupControl")
        );
    }

    #[test]
    fn table_from_dat_registers_collision_metadata() {
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                collision_group(0, LEFT_FLIPPER_GROUP_NAME),
                collision_group(1, PLUNGER_GROUP_NAME),
            ],
        };

        let table = PinballTable::from_dat(&dat_file);

        assert_eq!(table.collision_component_count(), 2);
        assert_eq!(table.collision_wall_count(), 5);
    }

    #[test]
    fn table_from_dat_registers_oneway_visual_geometry() {
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![oneway_group(0, "s_onewy1")],
        };

        let table = PinballTable::from_dat(&dat_file);

        assert_eq!(table.collision_wall_count(), 5);
    }

    #[test]
    fn table_from_dat_registers_kickout_visual_circle_geometry() {
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![kickout_group(0, "a_kout1")],
        };

        let table = PinballTable::from_dat(&dat_file);

        assert_eq!(table.collision_wall_count(), 4);
    }

    #[test]
    fn table_from_dat_uses_plunger_feed_position_for_ready_ball() {
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![plunger_group(0, PLUNGER_GROUP_NAME, 525.0, 315.0)],
        };
        let mut table = PinballTable::from_dat(&dat_file);

        table.dispatch(TableMessage::StartGame);

        let ball = table.active_ball().expect("start game should feed a ball");
        assert_eq!(ball.position.x, 525.0);
        assert_eq!(ball.position.y, 315.0);
        assert!(!ball.is_launched());
    }

    #[test]
    fn table_from_dat_applies_popup_target_timer_attribute() {
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![popup_target_group(0, "a_targ1", 0.35)],
        };
        let mut table = PinballTable::from_dat(&dat_file);

        table.dispatch(TableMessage::from_code(MessageCode::TPopupTargetDisable));
        table.dispatch(TableMessage::with_value(MessageCode::TPopupTargetEnable, 0.0));
        table.tick_components(0.34);
        let target = table.find_component("a_targ1").expect("popup target should exist");
        assert!(!target.is_active());

        table.tick_components(0.01);
        let target = table.find_component("a_targ1").expect("popup target should exist");
        assert!(target.is_active());
    }

    #[test]
    fn table_from_dat_applies_light_bargraph_timer_attribute() {
        let dat_file = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![light_bargraph_group(0, FUEL_BARGRAPH_GROUP_NAME, &[0.2, 0.1, 0.05, 0.05])],
        };
        let mut table = PinballTable::from_dat(&dat_file);

        table.dispatch(TableMessage::with_value(MessageCode::TLightGroupToggleSplitIndex, 3.0));
        let bargraph = table
            .find_component(FUEL_BARGRAPH_GROUP_NAME)
            .expect("fuel bargraph should exist");
        assert_eq!(bargraph.state().message_field, 3);

        table.tick_components(0.05);
        let bargraph = table
            .find_component(FUEL_BARGRAPH_GROUP_NAME)
            .expect("fuel bargraph should exist");
        assert_eq!(bargraph.state().message_field, 2);
    }
}
