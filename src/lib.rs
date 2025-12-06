use std::{collections::HashSet, sync::mpsc};

use common_game::{
    components::{
        planet::{Planet, PlanetAI, PlanetState, PlanetType},
        resource::{
            BasicResource, BasicResourceType, Carbon, Combinator, ComplexResource,
            ComplexResourceRequest, ComplexResourceType, Generator, GenericResource,
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
        if !self.enabled {
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
                        planet_state: state.to_dummy(),
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
        if !self.enabled {
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
                    msg,
                } => {
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
                    Some(PlanetToExplorer::CombineResourceResponse {
                        complex_response: Err((
                            String::from("Not supported"),
                            resource1,
                            resource2,
                        )),
                    })
                }
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
        if !self.enabled {
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

#[cfg(test)]
mod test {
    use super::*;
    use common_game::components::forge::Forge;
    use lazy_static::lazy_static;
    use std::sync::mpsc::{Receiver, Sender};
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
        let (tx_rx_orch_planet, rx_orch_planet) = mpsc::channel();
        let (tx_planet_orch, rx_planet_orch) = mpsc::channel();
        let (tx_expl_planet, rx_expl_planet) = mpsc::channel();
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
        let (tx_orchestrator_to_planet, rx_orchestrator_to_planet) = mpsc::channel();
        let (tx_planet_to_orchestrator, rx_planet_to_orchestrator) = mpsc::channel();
        let (_, rx_explorer_to_planet) = mpsc::channel();
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
        /* let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
        ));  FIX: Shouldn't we receive an acknowlegment? */
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
        let (tx_orchestrator_to_planet, rx_orchestrator_to_planet) = mpsc::channel();
        let (tx_planet_to_orchestrator, rx_planet_to_orchestrator) = mpsc::channel();
        let (tx_explorer_to_planet, rx_explorer_to_planet) = mpsc::channel();
        let (tx_planet_to_explorer, rx_planet_to_explorer) = mpsc::channel();
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
        /* let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
        ));  FIX: Shouldn't we receive an acknowlegment? */
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
