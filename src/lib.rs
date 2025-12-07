use std::collections::HashSet;

use crossbeam_channel::{Sender, Receiver};

use common_game::{
    components::{
        planet::{Planet, PlanetAI, PlanetState, PlanetType},
        resource::{
            BasicResource, BasicResourceType, Carbon, Combinator, ComplexResource,
            ComplexResourceRequest, ComplexResourceType, Generator, GenericResource,
        },
        rocket::Rocket,
    },
    logging::{ActorType, Channel, EventType, LogEvent, Payload},
    protocols::messages::{
        ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
    },
};

macro_rules! log_event {
    ($source_actor:expr, $source_id:expr, $dest_actor:expr, $dest_id:expr, $event_type:expr, $channel:expr, $payload_key:expr, $payload_value:expr) => {
        LogEvent::new(
            $source_actor,
            $source_id,
            $dest_actor,
            $dest_id,
            $event_type,
            $channel,
            Payload::from([($payload_key, $payload_value)]),
        )
        .emit();
    };
}

/// Function to create an instance of our migthy and glorious industrial planet Carbonium.
/// Just pass your id and comunication channels to get started.
#[must_use]
pub fn create_carbonium(
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
            match msg {
                OrchestratorToPlanet::Sunray(sunray) => {
                    if state.has_rocket() {
                        // Focus on building Carbon for storage.
                        if let Some((energy_cell, _)) = state.empty_cell() {
                            // Charge the EnergyCell.
                            energy_cell.charge(sunray);
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Orchestrator,
                                String::from("Master"),
                                EventType::MessagePlanetToOrchestrator,
                                Channel::Debug,
                                String::from("SunrayAck"),
                                String::from(
                                    "Has Rocket and Uncharged EnergyCell found. Sunray consumed to charge an EnergyCell.",
                                )
                            );
                        } else if let Some((energy_cell, _)) = state.full_cell()
                            && let Ok(carbon) = generator.make_carbon(energy_cell)
                        {
                            // Consume the EnergyCell to produce Carbon for storage and charge it.
                            self.storage.push(carbon);
                            energy_cell.charge(sunray);
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Orchestrator,
                                String::from("Master"),
                                EventType::MessagePlanetToOrchestrator,
                                Channel::Debug,
                                String::from("SunrayAck"),
                                String::from(
                                    "Has Rocket and Uncharged EnergyCell not found. Generated Carbon and stored it then consumed the Sunray to charge an EnergyCell.",
                                )
                            );
                        } else {
                            // Something went wrong.
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Orchestrator,
                                String::from("Master"),
                                EventType::MessagePlanetToOrchestrator,
                                Channel::Error,
                                String::from("SunrayAck"),
                                String::from(
                                    "No Rocket and Uncharged EnergyCell found and not found. This should't ever happen.",
                                )
                            );
                        }
                    } else {
                        // Focus on building a Rocket.
                        if let Some((energy_cell, i)) = state.empty_cell() {
                            // Charge the EnergyCell and consume it.
                            energy_cell.charge(sunray);
                            let _ = state.build_rocket(i);
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Orchestrator,
                                String::from("Master"),
                                EventType::MessagePlanetToOrchestrator,
                                Channel::Debug,
                                String::from("SunrayAck"),
                                String::from(
                                    "No Rocket and Uncharged EnergyCell found. Sunray consumed to charge an EnergyCell then build a Rocket.",
                                )
                            );
                        } else if let Some((_, i)) = state.full_cell() {
                            // Consume the EnergyCell and charge it.
                            let _ = state.build_rocket(i);
                            if let Some((energy_cell, _)) = state.empty_cell() {
                                energy_cell.charge(sunray);
                                log_event!(
                                    ActorType::Planet,
                                    state.id(),
                                    ActorType::Orchestrator,
                                    String::from("Master"),
                                    EventType::MessagePlanetToOrchestrator,
                                    Channel::Debug,
                                    String::from("SunrayAck"),
                                    String::from(
                                        "No Rocket and Uncharged EnergyCell not found. Built a Rocket then consumed the Sunray to charge an EnergyCell.",
                                    )
                                );
                            } else {
                                // Something went wrong.
                                log_event!(
                                    ActorType::Planet,
                                    state.id(),
                                    ActorType::Orchestrator,
                                    String::from("Master"),
                                    EventType::MessagePlanetToOrchestrator,
                                    Channel::Error,
                                    String::from("SunrayAck"),
                                    String::from(
                                        "No Rocket and Uncharged EnergyCell found and not found. This should't ever happen.",
                                    )
                                );
                            }
                        } else {
                            // Something went wrong.
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Orchestrator,
                                String::from("Master"),
                                EventType::MessagePlanetToOrchestrator,
                                Channel::Error,
                                String::from("SunrayAck"),
                                String::from(
                                    "No Rocket and Uncharged EnergyCell found and not found. This should't ever happen.",
                                )
                            );
                        }
                    }
                    Some(PlanetToOrchestrator::SunrayAck {
                        planet_id: state.id(),
                    })
                }
                OrchestratorToPlanet::InternalStateRequest => {
                    log_event!(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Orchestrator,
                        String::from("Master"),
                        EventType::MessagePlanetToOrchestrator,
                        Channel::Debug,
                        String::from("PlanetStateResponse"),
                        String::from("PlanetState returned.")
                    );

                    Some(PlanetToOrchestrator::InternalStateResponse {
                        planet_id: state.id(),
                        planet_state: state.to_dummy(),
                    })
                }
                _ => {
                    log_event!(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Orchestrator,
                        String::from("Master"),
                        EventType::MessageOrchestratorToPlanet,
                        Channel::Error,
                        String::from("OrchestratorMessageHandler"),
                        String::from("Unexpected type of message received.")
                    );
                    None
                }
            }
        } else {
            log_event!(
                ActorType::Planet,
                state.id(),
                ActorType::Orchestrator,
                String::from("Master"),
                EventType::MessagePlanetToOrchestrator,
                Channel::Info,
                String::from("PlanetAI"),
                String::from("PlanetAI is stopped.")
            );

            None
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
            match msg {
                ExplorerToPlanet::SupportedResourceRequest { explorer_id } => {
                    log_event!(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Explorer,
                        format!("{explorer_id}"),
                        EventType::MessagePlanetToExplorer,
                        Channel::Debug,
                        String::from("BasicResourcesResponse"),
                        format!("{:?}", Self::BASIC_RESOURCES)
                    );

                    Some(PlanetToExplorer::SupportedResourceResponse {
                        resource_list: HashSet::from(Self::BASIC_RESOURCES),
                    })
                }
                ExplorerToPlanet::SupportedCombinationRequest { explorer_id } => {
                    log_event!(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Explorer,
                        format!("{explorer_id}"),
                        EventType::MessagePlanetToExplorer,
                        Channel::Debug,
                        String::from("ComplexResourcesResponse"),
                        String::from("No ComplexResource supported.")
                    );

                    Some(PlanetToExplorer::SupportedCombinationResponse {
                        combination_list: HashSet::new(),
                    })
                }

                ExplorerToPlanet::GenerateResourceRequest {
                    explorer_id,
                    resource,
                } => {
                    if resource == BasicResourceType::Carbon {
                        // If there is Carbon in the storage return that otherwise try to build it.
                        if let Some(carbon) = self.storage.pop() {
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Explorer,
                                format!("{explorer_id}"),
                                EventType::MessagePlanetToExplorer,
                                Channel::Debug,
                                String::from("GenerateResourceResponse"),
                                String::from("Carbon popped from storage.")
                            );

                            Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(BasicResource::Carbon(carbon)),
                            })
                        } else if let Some((energy_cell, _)) = state.full_cell()
                            && let Ok(carbon) = generator.make_carbon(energy_cell)
                        {
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Explorer,
                                format!("{explorer_id}"),
                                EventType::MessagePlanetToExplorer,
                                Channel::Debug,
                                String::from("GenerateResourceResponse"),
                                String::from("Carbon created from EnergyCell.")
                            );

                            Some(PlanetToExplorer::GenerateResourceResponse {
                                resource: Some(BasicResource::Carbon(carbon)),
                            })
                        } else {
                            log_event!(
                                ActorType::Planet,
                                state.id(),
                                ActorType::Explorer,
                                format!("{explorer_id}"),
                                EventType::MessagePlanetToExplorer,
                                Channel::Debug,
                                String::from("GenerateResourceResponse"),
                                String::from("Storage is empty and no EnergyCell is charged.")
                            );

                            Some(PlanetToExplorer::GenerateResourceResponse { resource: None })
                        }
                    } else {
                        log_event!(
                            ActorType::Planet,
                            state.id(),
                            ActorType::Explorer,
                            format!("{explorer_id}"),
                            EventType::MessagePlanetToExplorer,
                            Channel::Debug,
                            String::from("GenerateResourceResponse"),
                            format!("{resource:?} not supported.")
                        );

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

                    log_event!(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Explorer,
                        format!("{explorer_id}"),
                        EventType::MessagePlanetToExplorer,
                        Channel::Debug,
                        String::from("CombineResourceResponse"),
                        String::from("ComplexResources are not supported.")
                    );

                    Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Err((
                            String::from("Not supported"),
                            resource1,
                            resource2,
                        )),
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

                    log_event!(
                        ActorType::Planet,
                        state.id(),
                        ActorType::Explorer,
                        format!("{explorer_id}"),
                        EventType::MessagePlanetToExplorer,
                        Channel::Debug,
                        String::from("AvailableEnergyCellResponse"),
                        format!(
                            "{}/{} EnergyCells are charged.",
                            charged_cells,
                            state.cells_count()
                        )
                    );

                    Some(PlanetToExplorer::AvailableEnergyCellResponse {
                        available_cells: charged_cells,
                    })
                }
            }
        } else {
            log_event!(
                ActorType::Planet,
                state.id(),
                ActorType::Explorer,
                String::from("Any"),
                EventType::MessagePlanetToExplorer,
                Channel::Info,
                String::from("PlanetAI"),
                String::from("PlanetAI is stopped.")
            );

            None
        }
    }

    fn handle_asteroid(
        &mut self,
        state: &mut PlanetState,
        _: &Generator,
        _: &Combinator,
    ) -> Option<Rocket> {
        if !self.enabled {
            log_event!(
                ActorType::Planet,
                state.id(),
                ActorType::Orchestrator,
                String::from("Master"),
                EventType::MessagePlanetToOrchestrator,
                Channel::Info,
                String::from("PlanetAI"),
                String::from("PlanetAI is stopped.")
            );

            None
        } else if state.has_rocket() {
            log_event!(
                ActorType::Planet,
                state.id(),
                ActorType::Orchestrator,
                String::from("Master"),
                EventType::MessagePlanetToOrchestrator,
                Channel::Debug,
                String::from("AsteroidAck"),
                String::from("Rocket was ready and used.")
            );

            state.take_rocket()
        } else {
            // If I don't have a Rocket right now, try to build it.
            if let Some((_, i)) = state.full_cell() {
                let _ = state.build_rocket(i);
                log_event!(
                    ActorType::Planet,
                    state.id(),
                    ActorType::Orchestrator,
                    String::from("Master"),
                    EventType::MessagePlanetToOrchestrator,
                    Channel::Debug,
                    String::from("AsteroidAck"),
                    String::from("EnergyCell used to build Rocket and then used.")
                );

                state.take_rocket()
            } else {
                log_event!(
                    ActorType::Planet,
                    state.id(),
                    ActorType::Orchestrator,
                    String::from("Master"),
                    EventType::MessagePlanetToOrchestrator,
                    Channel::Debug,
                    String::from("AsteroidAck"),
                    String::from(
                        "No Rocket or charged EnergyCell is available. The Planet will be destroyed."
                    )
                );
                
                None
            }
        }
    }

    fn start(&mut self, state: &PlanetState) {
        log_event!(
            ActorType::Planet,
            state.id(),
            ActorType::Orchestrator,
            String::from("Master"),
            EventType::MessagePlanetToOrchestrator,
            Channel::Debug,
            String::from("PlanetAI"),
            String::from("PlanetAI is now starting.")
        );
        self.enabled = true;
    }

    fn stop(&mut self, state: &PlanetState) {
        log_event!(
            ActorType::Planet,
            state.id(),
            ActorType::Orchestrator,
            String::from("Master"),
            EventType::MessagePlanetToOrchestrator,
            Channel::Debug,
            String::from("PlanetAI"),
            String::from("PlanetAI is now stopping.")
        );
        self.enabled = false;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use common_game::components::{forge::Forge};
    use lazy_static::lazy_static;
    use crossbeam_channel::{Sender, Receiver};
    use std::thread;

    lazy_static! {
        static ref FORGE: Forge = Forge::new().unwrap();
    }

    fn build_planet() -> (
        Planet,
        Sender<OrchestratorToPlanet>,
        Receiver<PlanetToOrchestrator>,
        Sender<ExplorerToPlanet>,
    ) {
        let (tx_rx_orch_planet, rx_orch_planet) = crossbeam_channel::unbounded();
        let (tx_planet_orch, rx_planet_orch) = crossbeam_channel::unbounded();
        let (tx_expl_planet, rx_expl_planet) = crossbeam_channel::unbounded();
        let planet = create_carbonium(1, rx_orch_planet, tx_planet_orch, rx_expl_planet);
        (planet, tx_rx_orch_planet, rx_planet_orch, tx_expl_planet)
    }

    #[test]
    fn test_create_carbonium() {
        let (planet, ..) = build_planet();
        // PlanetType doesn't implement PartialEq, so we can't compare it directly like this:
        //      planet.planet_type() == PlanetType::A;
        match planet.planet_type() {
            PlanetType::A => { /* Test passed */ }
            _ => panic!("Expected PlanetType::A, found {:?}", planet.planet_type()),
        }
    }

    #[test]
    fn test_carbonium_basic_resources() {
        let basic_resources: HashSet<BasicResourceType> = HashSet::from(Carbonium::BASIC_RESOURCES);
        assert!(basic_resources.contains(&BasicResourceType::Carbon));
        assert_eq!(basic_resources.len(), 1);
    }

    #[test]
    fn test_carbonium_complex_resources() {
        let complex_resources: HashSet<ComplexResourceType> =
            HashSet::from(Carbonium::COMPLEX_RESOURCES);
        assert!(complex_resources.is_empty());
    }

    #[test]
    fn test_carbonium_ai_initial_state() {
        let ai = Carbonium::AI;
        assert!(!ai.enabled);
        assert!(ai.storage.is_empty());
    }

    #[test]
    fn test_carbonium_ai_enable_disable() {
        let (planet, ..) = build_planet();
        let state = planet.state();
        let mut ai = Carbonium::AI;
        ai.start(&state);
        assert!(ai.enabled);
        ai.stop(&state);
        assert!(!ai.enabled);
    }

    #[test]
    fn orchestrator_to_planet_start() {
        todo!()
    }

    #[test]
    fn orchestrator_to_planet_stop() {
        todo!()
    }

    #[test]
    fn orchestrator_to_planet_state_request() {
        let (tx_orchestrator_to_planet, rx_orchestrator_to_planet) = crossbeam_channel::unbounded();
        let (tx_planet_to_orchestrator, rx_planet_to_orchestrator) = crossbeam_channel::unbounded();
        let (_, rx_explorer_to_planet) = crossbeam_channel::unbounded();
        let mut carbonium = create_carbonium(
            0,
            rx_orchestrator_to_planet,
            tx_planet_to_orchestrator,
            rx_explorer_to_planet,
        );

        thread::spawn(move || carbonium.run());

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::StartPlanetAI)
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
        )); // FIX: Shouldn't we receive an acknowledgment?

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::InternalStateRequest)
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        if let Ok(PlanetToOrchestrator::InternalStateResponse {
            planet_id: 0,
            planet_state,
        }) = res
        {
            assert!(
                !planet_state.has_rocket && planet_state.charged_cells_count == 0,
                "The planet should be empty"
            );
        } else {
            panic!("Unexpected response from planet.");
        }
    }

    #[test]
    fn orchestrator_to_planet_sunray() {
        let (tx_orchestrator_to_planet, rx_orchestrator_to_planet) = crossbeam_channel::unbounded();
        let (tx_planet_to_orchestrator, rx_planet_to_orchestrator) = crossbeam_channel::unbounded();
        let (tx_explorer_to_planet, rx_explorer_to_planet) = crossbeam_channel::unbounded();
        let (tx_planet_to_explorer, rx_planet_to_explorer) = crossbeam_channel::unbounded();
        let mut carbonium = create_carbonium(
            0,
            rx_orchestrator_to_planet,
            tx_planet_to_orchestrator,
            rx_explorer_to_planet,
        );

        thread::spawn(move || carbonium.run());

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::StartPlanetAI)
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
        )); // FIX: Shouldn't we receive an acknowledgment?

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::Sunray(FORGE.generate_sunray()))
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::SunrayAck { planet_id: 0 })
        ));

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::InternalStateRequest)
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        if let Ok(PlanetToOrchestrator::InternalStateResponse {
            planet_id: 0,
            planet_state,
        }) = res
        {
            assert!(
                planet_state.has_rocket && planet_state.charged_cells_count == 0,
                "The first sunray received should be used to build a Rocket."
            );
        } else {
            panic!("Unexpected response from planet.");
        }

        for _ in 0..5 {
            tx_orchestrator_to_planet
                .send(OrchestratorToPlanet::Sunray(FORGE.generate_sunray()))
                .unwrap();

            let res = rx_planet_to_orchestrator.recv();
            assert!(matches!(
                res,
                Ok(PlanetToOrchestrator::SunrayAck { planet_id: 0 })
            ));
        }

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::InternalStateRequest)
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        if let Ok(PlanetToOrchestrator::InternalStateResponse {
            planet_id: 0,
            planet_state,
        }) = res
        {
            assert!(
                planet_state.charged_cells_count == planet_state.energy_cells.len(),
                "All EnergyCells should be charged."
            );
        } else {
            panic!("Unexpected response from planet.");
        }

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::Sunray(FORGE.generate_sunray()))
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::SunrayAck { planet_id: 0 })
        ));

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::IncomingExplorerRequest {
                explorer_id: 0,
                new_mpsc_sender: tx_planet_to_explorer,
            })
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::IncomingExplorerResponse {
                planet_id: 0,
                res: Ok(())
            })
        ));

        tx_explorer_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Carbon,
            })
            .unwrap();

        let res = rx_planet_to_explorer.recv();
        assert!(matches!(
            res,
            Ok(PlanetToExplorer::GenerateResourceResponse {
                resource: Some(BasicResource::Carbon(_))
            })
        ));

        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::InternalStateRequest)
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        if let Ok(PlanetToOrchestrator::InternalStateResponse {
            planet_id: 0,
            planet_state,
        }) = res
        {
            assert!(
                planet_state.charged_cells_count == planet_state.energy_cells.len(),
                "All EnergyCells are still charged because there was Carbon in storage."
            );
        } else {
            panic!("Unexpected response from planet.");
        }
    }

    #[test]
    fn explorer_to_planet_supported_basic_resources() {
        todo!()
    }

    #[test]
    fn explorer_to_planet_supported_combinations() {
        todo!()
    }

    #[test]
    fn explorer_to_planet_generate_resource() {
        todo!()
    }

    #[test]
    fn explorer_to_planet_combine_resource() {
        todo!()
    }

    #[test]
    fn explorer_to_planet_available_energy_cells() {
        todo!()
    }

    #[test]
    fn orchestrator_to_planet_asteroid() {
        todo!()
    }
}
