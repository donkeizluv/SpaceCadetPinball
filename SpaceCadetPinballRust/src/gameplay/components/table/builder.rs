use crate::assets::DatFile;
use crate::gameplay::components::group_name::{LEFT_FLIPPER_GROUP_NAME, PLUNGER_GROUP_NAME};
use crate::gameplay::components::{ComponentId, ComponentState, GameplayComponent};
use crate::gameplay::mechanics::{
    DrainMechanic, FlipperMechanic, PlaceholderMechanic, PlungerMechanic,
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
        .with_primary_group("a_roll179"),
        ComponentDefinition::new(
            ComponentId(32),
            "a_roll180",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover2Control")
        .with_primary_group("a_roll180"),
        ComponentDefinition::new(
            ComponentId(33),
            "a_roll181",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover3Control")
        .with_primary_group("a_roll181"),
        ComponentDefinition::new(
            ComponentId(34),
            "a_roll182",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover4Control")
        .with_primary_group("a_roll182"),
        ComponentDefinition::new(
            ComponentId(35),
            "a_roll183",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover5Control")
        .with_primary_group("a_roll183"),
        ComponentDefinition::new(
            ComponentId(36),
            "a_roll184",
            ComponentKind::Rollover,
            "TRollover",
        )
        .with_control("FuelRollover6Control")
        .with_primary_group("a_roll184"),
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
        ComponentKind::Blocker
        | ComponentKind::Gate
        | ComponentKind::Kickback
        | ComponentKind::Kickout
        | ComponentKind::Sink
        | ComponentKind::Oneway
        | ComponentKind::Rollover
        | ComponentKind::LightRollover
        | ComponentKind::Tripwire => Box::new(PlaceholderMechanic::from_state(state)),
    }
}

#[cfg(test)]
mod tests {
    use crate::assets::{DatFile, EntryData, EntryPayload, FieldType, GroupData};

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

    #[test]
    fn default_definitions_match_current_playable_slice() {
        let definitions = default_component_definitions();

        assert_eq!(definitions.len(), 41);
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

        assert_eq!(table.component_count(), 41);
        assert_eq!(table.link_report().component_count, 41);
        assert_eq!(table.link_report().resolved_group_count, 40);
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
    }
}
