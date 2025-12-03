#![allow(unused)]

use common_game::{
    components::{energy_cell, planet::*, resource::*, rocket::*},
    protocols::messages::*,
};
use std::sync::mpsc;
use std::time::SystemTime;

struct Carbonium {}
impl Carbonium {
    const PLANET_TYPE: PlanetType = PlanetType::A;
    const BASIC_RESOURCES: [BasicResourceType; 1] = [BasicResourceType::Carbon];
    const COMPLEX_RESOURCES: [ComplexResourceType; 0] = [];
    const AI: Self = Self {};
}
impl PlanetAI for Carbonium {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        match msg {
            OrchestratorToPlanet::Sunray(sunray) => {
                if let Some((i, energy_cell)) = state
                    .cells_iter_mut()
                    .enumerate()
                    .find(|(i, energy_cell)| !energy_cell.is_charged())
                {
                    energy_cell.charge(sunray);
                    if !state.has_rocket() {
                        // If I don't have a Rocket, use that EnergyCell
                        // immediately to build one.
                        state.build_rocket(i);
                    }
                }
                Some(PlanetToOrchestrator::SunrayAck {
                    planet_id: state.id(),
                    timestamp: SystemTime::now(),
                })
            }
            OrchestratorToPlanet::InternalStateRequest(internal_state_request_msg) => {
                Some(PlanetToOrchestrator::InternalStateResponse {
                    planet_id: state.id(),
                    planet_state: todo!(), // Requires ownership, but state is &mut. I think a reference should be passed.
                    timestamp: SystemTime::now(),
                })
            }
            _ => panic!("No other type of message should be received"), // Why separate into
                                                                        // multiple functions, tho?
        }
    }
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator, // I am not sure why these are arguments, and not defined by me.
        // What if the wrong Generator is passed?
        combinator: &Combinator, // Same as above.
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: todo!(), // Why does it need an Option<HashSet<BasicResourceType>>?
                                            // In Planet::new() asks for a Vec<BasicResourceType>. It's not consistent.
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: todo!(), // Why does it need an Option<ComplexResource>?
                                               // There are planets that support multiple complex resources, and why isn't
                                               // it ComplexResourceType?
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource,
            } => {
                if resource == BasicResourceType::Carbon {
                    if let Some(energy_cell) = state
                        .cells_iter_mut()
                        .find(|energy_cell| energy_cell.is_charged())
                    {
                        if let Ok(carbon) = generator.make_carbon(energy_cell) {
                            Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(BasicResource::Carbon(carbon)),
                            })
                        } else {
                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                    } else {
                        Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                    }
                } else {
                    Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                }
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: None,
                })
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: state.cells_iter().fold(0, |acc, energy_cell| {
                        if energy_cell.is_charged() {
                            acc + 1
                        } else {
                            acc
                        }
                    }),
                })
            }
            ExplorerToPlanet::InternalStateRequest { explorer_id } => {
                Some(PlanetToExplorer::InternalStateResponse {
                    planet_state: todo!(), // Requires ownership, but state is &mut. I think a reference should be passed.
                })
            }
        }
    }
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        combinator: &Combinator,
    ) -> Option<Rocket> {
        if state.has_rocket() {
            state.take_rocket()
        } else {
            // If I don't have a Rocket right now, try to build it.
            if let Some((i, energy_cell)) = state
                .cells_iter()
                .enumerate()
                .find(|(i, energy_cell)| energy_cell.is_charged())
            {
                state.build_rocket(i);
                state.take_rocket()
            } else {
                None
            }
        }
    }
    fn start(&mut self, state: &PlanetState) {
        // I am not sure what to put here, I could maybe spawn a thread that does something.
        todo!()
    }
    fn stop(&mut self, state: &PlanetState) {
        // I am not sure what to put here, I could kill the previusly mentioned thread.
        todo!()
    }
}

fn main() {
    // Example of usage
    let planet = Planet::new(
        0,
        Carbonium::PLANET_TYPE,
        Carbonium::AI,
        Carbonium::BASIC_RESOURCES.to_vec(),
        Carbonium::COMPLEX_RESOURCES.to_vec(),
        todo!(),
        todo!(),
    );
}
