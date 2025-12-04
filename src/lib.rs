use std::{collections::HashSet, sync::mpsc};

use common_game::{
    components::{
        planet::{Planet, PlanetAI, PlanetState, PlanetType},
        resource::{
            BasicResource, BasicResourceType, Carbon, Combinator, ComplexResourceType, Generator,
        },
        rocket::Rocket,
    },
    protocols::messages::{
        ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
    },
};

/// Function to create an instance of our migthy and glorious industrial planet Carbonium.
/// Just pass your id and comunication channels to get started.
#[must_use]
pub fn create_carbonium(
    id: u32,
    rx_orchestrator: mpsc::Receiver<OrchestratorToPlanet>,
    tx_orchestrator: mpsc::Sender<PlanetToOrchestrator>,
    rx_explorer: mpsc::Receiver<ExplorerToPlanet>,
    tx_explorer: mpsc::Sender<PlanetToExplorer>,
) -> Planet {
    Planet::new(
        id,
        Carbonium::PLANET_TYPE,
        Box::new(Carbonium::AI),
        Carbonium::BASIC_RESOURCES.to_vec(),
        Carbonium::COMPLEX_RESOURCES.to_vec(),
        (rx_orchestrator, tx_orchestrator),
        (rx_explorer, tx_explorer),
    )
    .unwrap() // This shouldn't ever panic because PlanetType requirements are always correct.
}

struct Carbonium {
    storage: Vec<Carbon>,
    enabled: bool,
}
impl Carbonium {
    const PLANET_TYPE: PlanetType = PlanetType::A;
    const BASIC_RESOURCES: [BasicResourceType; 1] = [BasicResourceType::Carbon];
    const COMPLEX_RESOURCES: [ComplexResourceType; 0] = [];
    const AI: Self = Self {
        storage: Vec::new(),
        enabled: false,
    };
}
impl PlanetAI for Carbonium {
    fn handle_orchestrator_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        _: &Combinator,
        msg: OrchestratorToPlanet,
    ) -> Option<PlanetToOrchestrator> {
        if self.enabled {
            None
        } else {
            match msg {
                OrchestratorToPlanet::Sunray(sunray) => {
                    if let Some((i, energy_cell)) = state
                        .cells_iter_mut()
                        .enumerate()
                        .find(|(_, energy_cell)| !energy_cell.is_charged())
                    {
                        energy_cell.charge(sunray);
                        if !state.has_rocket() {
                            // If I don't have a Rocket, use that EnergyCell
                            // immediately to build one.
                            let _ = state.build_rocket(i);
                        }
                    } else {
                        // If I can't charge an EnergyCell, consume one charge to build Carbon and
                        // store it, then charge that EnergyCell.
                        if let Some(energy_cell) = state
                            .cells_iter_mut()
                            .find(|energy_cell| energy_cell.is_charged())
                            && let Ok(carbon) = generator.make_carbon(energy_cell)
                        {
                            self.storage.push(carbon);
                            energy_cell.charge(sunray);
                        }
                    }
                    Some(PlanetToOrchestrator::SunrayAck {
                        planet_id: state.id(),
                    })
                }
                OrchestratorToPlanet::InternalStateRequest => {
                    Some(PlanetToOrchestrator::InternalStateResponse {
                        planet_id: state.id(),
                        planet_state: todo!(), // Requires ownership, but state is &mut. I think a reference should be passed.
                    })
                }
                _ => panic!("No other type of message should be received"), // Why separate into
                                                                            // multiple functions, tho?
            }
        }
    }
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator, // I am not sure why these are arguments, and not defined by me.
        // What if the wrong Generator is passed?
        _: &Combinator, // Same as above.
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        if self.enabled {
            None
        } else {
            match msg {
                ExplorerToPlanet::SupportedResourceRequest { explorer_id: _ } => {
                    Some(PlanetToExplorer::SupportedResourceResponse {
                        resource_list: HashSet::from(Self::BASIC_RESOURCES),
                    })
                }
                ExplorerToPlanet::SupportedCombinationRequest { explorer_id: _ } => {
                    Some(PlanetToExplorer::SupportedCombinationResponse {
                        combination_list: HashSet::new(),
                    })
                }

                ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id: _,
                    resource,
                } => {
                    if resource == BasicResourceType::Carbon {
                        // If there is Carbon in the storage return that otherwise try to build it.
                        if let Some(carbon) = self.storage.pop() {
                            Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(BasicResource::Carbon(carbon)),
                            })
                        } else if let Some(energy_cell) = state
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
                ExplorerToPlanet::CombineResourceRequest {
                    explorer_id: _,
                    msg: _,
                } => Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: None,
                }),
                ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: _ } => {
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
            }
        }
    }
    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator,
    ) -> Option<Rocket> {
        if self.enabled {
            None
        } else if state.has_rocket() {
            state.take_rocket()
        } else {
            // If I don't have a Rocket right now, try to build it.
            if let Some((i, _)) = state
                .cells_iter()
                .enumerate()
                .find(|(_, energy_cell)| energy_cell.is_charged())
            {
                let _ = state.build_rocket(i);
                state.take_rocket()
            } else {
                None
            }
        }
    }
    fn start(&mut self, _: &PlanetState) {
        self.enabled = true;
    }
    fn stop(&mut self, _: &PlanetState) {
        self.enabled = false;
    }
}
