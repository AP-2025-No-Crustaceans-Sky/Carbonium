#[cfg(test)]
mod tests;

use common_game::{
    components::{
        planet::{DummyPlanetState, Planet, PlanetAI, PlanetState, PlanetType},
        resource::{
            BasicResource, BasicResourceType, Combinator, ComplexResource, ComplexResourceRequest,
            ComplexResourceType, Generator, GenericResource,
        },
        rocket::Rocket,
    },
    logging::{ActorType, Channel, EventType, LogEvent, Participant, Payload},
    protocols::{
        orchestrator_planet::{OrchestratorToPlanet, PlanetToOrchestrator},
        planet_explorer::{ExplorerToPlanet, PlanetToExplorer},
    },
};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashSet;

macro_rules! payload {
    ($payload_key:expr, $payload_value:expr) => {
        Payload::from([(String::from($payload_key), String::from($payload_value))])
    };
}

/// Function to create an instance of our migthy and glorious industrial planet Carbonium.
/// Just pass your id and comunication channels to get started.
///
/// # Panics
/// This function has an unwrap, but it should never panic because the [`PlanetConstraints`] are
/// hardcoded to be correct.
///
/// Example:
/// ```
/// let (tx_orchestrator_to_planet, rx_orchestrator_to_planet) = crossbeam_channel::unbounded();
/// let (tx_planet_to_orchestrator, rx_planet_to_orchestrator) = crossbeam_channel::unbounded();
/// let (_, rx_explorer_to_planet) = crossbeam_channel::unbounded();
/// let carbonium = carbonium::create_planet(
///     0,
///     rx_orchestrator_to_planet,
///     tx_planet_to_orchestrator,
///     rx_explorer_to_planet,
/// );
/// assert!(matches!(carbonium.planet_type(), common_game::components::planet::PlanetType::A));
/// ```
#[must_use]
pub fn create_planet(
    id: u32,
    rx_orchestrator: Receiver<OrchestratorToPlanet>,
    tx_orchestrator: Sender<PlanetToOrchestrator>,
    rx_explorer: Receiver<ExplorerToPlanet>,
) -> Planet {
    Planet::new(
        id,
        Carbonium::PLANET_TYPE,
        Box::new(Carbonium::AI),
        Carbonium::BASIC_RESOURCES.to_vec(),
        Carbonium::COMPLEX_RESOURCES.to_vec(),
        (rx_orchestrator, tx_orchestrator),
        rx_explorer,
    )
    .unwrap() // This shouldn't ever panic because PlanetType requirements are always correct.
}

struct Carbonium {
    _private: (),
}

impl Carbonium {
    const PLANET_TYPE: PlanetType = PlanetType::A;
    const BASIC_RESOURCES: [BasicResourceType; 1] = [BasicResourceType::Carbon];
    const COMPLEX_RESOURCES: [ComplexResourceType; 0] = [];
    const AI: Self = Self { _private: () };
}

impl PlanetAI for Carbonium {
    fn handle_explorer_msg(
        &mut self,
        state: &mut PlanetState,
        generator: &Generator,
        _: &Combinator,
        msg: ExplorerToPlanet,
    ) -> Option<PlanetToExplorer> {
        let mut log_event = LogEvent::new(
            Some(Participant {
                actor_type: ActorType::Planet,
                id: state.id(),
            }),
            None,
            EventType::MessagePlanetToExplorer,
            Channel::Debug,
            payload!(String::new(), String::new()),
        );
        match msg {
            ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                log_event.receiver = Some(Participant {
                    actor_type: ActorType::Explorer,
                    id: explorer_id,
                });
                log_event.payload = payload!(
                    "BasicResourcesResponse",
                    format!("{:?}", Self::BASIC_RESOURCES)
                );
                log_event.emit();
                Some(PlanetToExplorer::SupportedResourceResponse {
                    resource_list: HashSet::from(Self::BASIC_RESOURCES),
                })
            }
            ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                log_event.receiver = Some(Participant {
                    actor_type: ActorType::Explorer,
                    id: explorer_id,
                });
                log_event.payload =
                    payload!("ComplexResourcesResponse", "No ComplexResource supported.");
                log_event.emit();
                Some(PlanetToExplorer::SupportedCombinationResponse {
                    combination_list: HashSet::new(),
                })
            }
            ExplorerToPlanet::GenerateResourceRequest {
                explorer_id,
                resource,
            } => {
                if resource != BasicResourceType::Carbon {
                    log_event.receiver = Some(Participant {
                        actor_type: ActorType::Explorer,
                        id: explorer_id,
                    });
                    log_event.payload = payload!(
                        "GenerateResourceResponse",
                        format!("{resource:?} not supported.")
                    );
                    log_event.emit();
                    return Some(PlanetToExplorer::GenerateResourceResponse { resource: None });
                }
                if !state.has_rocket() && state.to_dummy().charged_cells_count < 2 {
                    log_event.receiver = Some(Participant {
                        actor_type: ActorType::Explorer,
                        id: explorer_id,
                    });
                    log_event.payload = payload!(
                        "GenerateResourceResponse",
                        format!("Planet needs EnergyCells to build an emergency Rocket.")
                    );
                    log_event.emit();
                    return Some(PlanetToExplorer::GenerateResourceResponse { resource: None });
                }
                if let Some((energy_cell, _)) = state.full_cell()
                    && let Ok(carbon) = generator.make_carbon(energy_cell)
                {
                    log_event.receiver = Some(Participant {
                        actor_type: ActorType::Explorer,
                        id: explorer_id,
                    });
                    log_event.payload = payload!(
                        "GenerateResourceResponse",
                        "Carbon created from EnergyCell."
                    );
                    log_event.emit();
                    Some(PlanetToExplorer::GenerateResourceResponse {
                        resource: Some(BasicResource::Carbon(carbon)),
                    })
                } else {
                    log_event.receiver = Some(Participant {
                        actor_type: ActorType::Explorer,
                        id: explorer_id,
                    });
                    log_event.payload = payload!(
                        "GenerateResourceResponse",
                        "Storage is empty and no EnergyCell is charged."
                    );
                    log_event.emit();
                    Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                }
            }
            ExplorerToPlanet::CombineResourceRequest { explorer_id, msg } => {
                let (resource1, resource2): (GenericResource, GenericResource) = match msg {
                    ComplexResourceRequest::Water(hydrogen, oxygen) => (
                        GenericResource::BasicResources(BasicResource::Hydrogen(hydrogen)),
                        GenericResource::BasicResources(BasicResource::Oxygen(oxygen)),
                    ),
                    ComplexResourceRequest::Diamond(carbon, carbon1) => (
                        GenericResource::BasicResources(BasicResource::Carbon(carbon)),
                        GenericResource::BasicResources(BasicResource::Carbon(carbon1)),
                    ),
                    ComplexResourceRequest::Life(water, carbon) => (
                        GenericResource::ComplexResources(ComplexResource::Water(water)),
                        GenericResource::BasicResources(BasicResource::Carbon(carbon)),
                    ),
                    ComplexResourceRequest::Robot(silicon, life) => (
                        GenericResource::BasicResources(BasicResource::Silicon(silicon)),
                        GenericResource::ComplexResources(ComplexResource::Life(life)),
                    ),
                    ComplexResourceRequest::Dolphin(water, life) => (
                        GenericResource::ComplexResources(ComplexResource::Water(water)),
                        GenericResource::ComplexResources(ComplexResource::Life(life)),
                    ),
                    ComplexResourceRequest::AIPartner(robot, diamond) => (
                        GenericResource::ComplexResources(ComplexResource::Robot(robot)),
                        GenericResource::ComplexResources(ComplexResource::Diamond(diamond)),
                    ),
                };
                log_event.receiver = Some(Participant {
                    actor_type: ActorType::Explorer,
                    id: explorer_id,
                });
                log_event.payload = payload!(
                    "CombineResourceResponse",
                    "ComplexResources are not supported."
                );
                log_event.emit();
                Some(PlanetToExplorer::CombineResourceResponse {
                    complex_response: Err((String::from("Not supported"), resource1, resource2)),
                })
            }
            ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id } => {
                let charged_cells = state.cells_iter().fold(0, |acc, energy_cell| {
                    if energy_cell.is_charged() {
                        acc + 1
                    } else {
                        acc
                    }
                });
                log_event.receiver = Some(Participant {
                    actor_type: ActorType::Explorer,
                    id: explorer_id,
                });
                log_event.payload = payload!(
                    "AvailableEnergyCellResponse",
                    format!(
                        "{}/{} EnergyCells are charged.",
                        charged_cells,
                        state.cells_count()
                    )
                );
                log_event.emit();
                Some(PlanetToExplorer::AvailableEnergyCellResponse {
                    available_cells: charged_cells,
                })
            }
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator,
    ) -> Option<Rocket> {
        let mut log_event = LogEvent::new(
            Some(Participant {
                actor_type: ActorType::Planet,
                id: state.id(),
            }),
            Some(Participant {
                actor_type: ActorType::Orchestrator,
                id: 0,
            }),
            EventType::MessagePlanetToOrchestrator,
            Channel::Debug,
            payload!(String::new(), String::new()),
        );
        if state.has_rocket() {
            log_event.payload = payload!("AsteroidAck", "Rocket was ready and used.");
            log_event.emit();
            state.take_rocket()
        } else {
            // If I don't have a Rocket right now, try to build it.
            if let Some((_, i)) = state.full_cell() {
                let _ = state.build_rocket(i);
                log_event.payload = payload!(
                    "AsteroidAck",
                    "EnergyCell used to build Rocket and then used."
                );
                log_event.emit();
                state.take_rocket()
            } else {
                log_event.payload = payload!(
                    "AsteroidAck",
                    "No Rocket or charged EnergyCell is available. The Planet will be \
                     destroyed."
                );
                log_event.emit();
                None
            }
        }
    }

    fn handle_sunray(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator,
        sunray: common_game::components::sunray::Sunray,
    ) {
        let mut log_event = LogEvent::new(
            Some(Participant {
                actor_type: ActorType::Planet,
                id: state.id(),
            }),
            Some(Participant {
                actor_type: ActorType::Orchestrator,
                id: 0,
            }),
            EventType::MessagePlanetToOrchestrator,
            Channel::Debug,
            payload!(String::new(), String::new()),
        );
        if state.has_rocket() {
            // Focus on building Carbon for storage.
            if let Some((energy_cell, _)) = state.empty_cell() {
                // Charge the EnergyCell.
                energy_cell.charge(sunray);
                log_event.payload = payload!(
                    "SunrayAck",
                    "Has Rocket and Uncharged EnergyCell found. Sunray consumed to \
                     charge an EnergyCell."
                );
                log_event.emit();
            }
        } else {
            // Focus on building a Rocket.
            if let Some((energy_cell, i)) = state.empty_cell() {
                // Charge the EnergyCell and consume it.
                energy_cell.charge(sunray);
                let _ = state.build_rocket(i);
                log_event.payload = payload!(
                    "SunrayAck",
                    "No Rocket and Uncharged EnergyCell found. Sunray consumed to \
                     charge an EnergyCell then build a Rocket."
                );
                log_event.emit();
            } else if let Some((_, i)) = state.full_cell() {
                // Consume the EnergyCell and charge it.
                let _ = state.build_rocket(i);
                if let Some((energy_cell, _)) = state.empty_cell() {
                    energy_cell.charge(sunray);
                    log_event.payload = payload!(
                        "SunrayAck",
                        "No Rocket and Uncharged EnergyCell not found. Built a Rocket \
                         then consumed the Sunray to charge an EnergyCell."
                    );
                    log_event.emit();
                } else {
                    // Something went wrong.
                    log_event.channel = Channel::Error;
                    log_event.payload = payload!(
                        "SunrayAck",
                        "No Rocket and Uncharged EnergyCell found and not found. This \
                         should't ever happen."
                    );
                    log_event.emit();
                }
            } else {
                // Something went wrong.
                log_event.channel = Channel::Error;
                log_event.payload = payload!(
                    "SunrayAck",
                    "No Rocket and Uncharged EnergyCell found and not found. This \
                     should't ever happen."
                );
                log_event.emit();
            }
        }
    }

    fn handle_internal_state_req(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator,
    ) -> DummyPlanetState {
        let mut log_event = LogEvent::new(
            Some(Participant {
                actor_type: ActorType::Planet,
                id: state.id(),
            }),
            Some(Participant {
                actor_type: ActorType::Orchestrator,
                id: 0,
            }),
            EventType::MessagePlanetToOrchestrator,
            Channel::Debug,
            payload!(String::new(), String::new()),
        );
        log_event.payload = payload!("PlanetStateResponse", "PlanetState returned.");
        log_event.emit();
        state.to_dummy()
    }
}
