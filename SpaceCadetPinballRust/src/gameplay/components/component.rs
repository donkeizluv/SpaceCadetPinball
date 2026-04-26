use super::group::ComponentId;
use super::messages::TableMessage;
use super::table::{SimulationState, TableInputState};

pub trait GameplayComponent {
    fn id(&self) -> ComponentId;

    fn name(&self) -> &str;

    fn on_message(
        &mut self,
        message: TableMessage,
        simulation: &mut SimulationState,
        table_state: &TableInputState,
    );

    fn tick(
        &mut self,
        _simulation: &mut SimulationState,
        _table_state: &TableInputState,
        _dt: f32,
    ) {
    }
}
