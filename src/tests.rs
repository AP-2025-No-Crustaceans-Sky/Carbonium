use crate::{Carbonium, create_carbonium};
use common_game::components::forge::Forge;
use common_game::{
    components::resource::{
        BasicResource, BasicResourceType, ComplexResourceRequest, ComplexResourceType,
        GenericResource,
    },
    protocols::messages::{
        ExplorerToPlanet, OrchestratorToPlanet, PlanetToExplorer, PlanetToOrchestrator,
    },
};
use lazy_static::lazy_static;
use ntest::timeout;
use std::collections::HashSet;
use std::thread;

lazy_static! {
    static ref FORGE: Forge = Forge::new().unwrap();
}

#[test]
fn carbonium_basic_resources() {
    let basic_resources: HashSet<BasicResourceType> = HashSet::from(Carbonium::BASIC_RESOURCES);
    assert!(basic_resources.contains(&BasicResourceType::Carbon));
    assert_eq!(basic_resources.len(), 1);
}

#[test]
fn carbonium_complex_resources() {
    let complex_resources: HashSet<ComplexResourceType> =
        HashSet::from(Carbonium::COMPLEX_RESOURCES);
    assert!(complex_resources.is_empty());
}

#[test]
fn carbonium_ai_initial_state() {
    let ai = Carbonium::AI;
    assert!(ai.storage.is_empty());
}

#[test]
fn orchestrator_to_planet_start() {
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
        .send(OrchestratorToPlanet::InternalStateRequest)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(
        matches!(res, Ok(PlanetToOrchestrator::Stopped { planet_id: 0 })),
        "Planet is stopped."
    );

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::StartPlanetAI)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(
        matches!(
            res,
            Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
        ),
        "Planet should be started."
    );

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::InternalStateRequest)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(
        matches!(
            res,
            Ok(PlanetToOrchestrator::InternalStateResponse {
                planet_id: _,
                planet_state: _
            })
        ),
        "Planet is running."
    );
}

#[test]
fn orchestrator_to_planet_stop() {
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
    assert!(
        matches!(
            res,
            Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
        ),
        "Planet should be started."
    );

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::StopPlanetAI)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(
        matches!(
            res,
            Ok(PlanetToOrchestrator::StopPlanetAIResult { planet_id: 0 })
        ),
        "Planet should be stopped."
    );

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::InternalStateRequest)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(
        matches!(res, Ok(PlanetToOrchestrator::Stopped { planet_id: 0 })),
        "Planet is stopped."
    );
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
            !planet_state.has_rocket && planet_state.charged_cells_count == 0,
            "The planet should be empty."
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
    ));

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

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::Asteroid(FORGE.generate_asteroid()))
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(matches!(
        res,
        Ok(PlanetToOrchestrator::AsteroidAck {
            planet_id: 0,
            rocket: Some(_)
        })
    ));

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
            planet_state.has_rocket
                && planet_state.charged_cells_count == planet_state.energy_cells.len(),
            "The first sunray received should be used to build a Rocket."
        );
    } else {
        panic!("Unexpected response from planet.");
    }
}

#[test]
fn explorer_to_planet_supported_basic_resources() {
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
        .send(ExplorerToPlanet::SupportedResourceRequest { explorer_id: 0 })
        .unwrap();

    let res = rx_planet_to_explorer.recv();
    if let Ok(PlanetToExplorer::SupportedResourceResponse { resource_list }) = res {
        assert_eq!(
            resource_list,
            HashSet::from([BasicResourceType::Carbon]),
            "Only Carbon is supported."
        );
    } else {
        panic!("Unexpected response from planet.");
    }
}

#[test]
fn explorer_to_planet_supported_combinations() {
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
        .send(ExplorerToPlanet::SupportedCombinationRequest { explorer_id: 0 })
        .unwrap();

    let res = rx_planet_to_explorer.recv();
    if let Ok(PlanetToExplorer::SupportedCombinationResponse { combination_list }) = res {
        assert_eq!(
            combination_list,
            HashSet::new(),
            "No ComplexResource is supported."
        );
    } else {
        panic!("Unexpected response from planet.");
    }
}

#[test]
fn explorer_to_planet_generate_resource() {
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
        Ok(PlanetToExplorer::GenerateResourceResponse { resource: None })
    ));

    for _ in 0..2 {
        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::Sunray(FORGE.generate_sunray()))
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::SunrayAck { planet_id: 0 })
        ));
    }

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

    let other_resources = [
        BasicResourceType::Hydrogen,
        BasicResourceType::Silicon,
        BasicResourceType::Oxygen,
    ];
    for item in other_resources {
        tx_explorer_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: item,
            })
            .unwrap();

        let res = rx_planet_to_explorer.recv();
        assert!(matches!(
            res,
            Ok(PlanetToExplorer::GenerateResourceResponse { resource: None })
        ));
    }
}

#[test]
fn explorer_to_planet_combine_resource() {
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
    ));

    for _ in 0..3 {
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

    let mut carbons = Vec::new();
    for _ in 0..2 {
        tx_explorer_to_planet
            .send(ExplorerToPlanet::GenerateResourceRequest {
                explorer_id: 0,
                resource: BasicResourceType::Carbon,
            })
            .unwrap();

        let res = rx_planet_to_explorer.recv();
        if let Ok(PlanetToExplorer::GenerateResourceResponse {
            resource: Some(BasicResource::Carbon(carbon)),
        }) = res
        {
            carbons.push(carbon);
        }
    }

    tx_explorer_to_planet
        .send(ExplorerToPlanet::CombineResourceRequest {
            explorer_id: 0,
            msg: ComplexResourceRequest::Diamond(carbons.pop().unwrap(), carbons.pop().unwrap()),
        })
        .unwrap();

    let res = rx_planet_to_explorer.recv();
    assert!(matches!(
        res,
        Ok(PlanetToExplorer::CombineResourceResponse {
            complex_response: Err((
                _,
                GenericResource::BasicResources(BasicResource::Carbon(_)),
                GenericResource::BasicResources(BasicResource::Carbon(_))
            ))
        })
    ));
}

#[test]
fn explorer_to_planet_available_energy_cells() {
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
    ));

    for _ in 0..3 {
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
        .send(ExplorerToPlanet::AvailableEnergyCellRequest { explorer_id: 0 })
        .unwrap();

    let res = rx_planet_to_explorer.recv();
    assert!(matches!(
        res,
        Ok(PlanetToExplorer::AvailableEnergyCellResponse { available_cells: 2 })
    ));
}

#[test]
#[timeout(500)]
fn orchestrator_to_planet_asteroid() {
    let (tx_orchestrator_to_planet, rx_orchestrator_to_planet) = crossbeam_channel::unbounded();
    let (tx_planet_to_orchestrator, rx_planet_to_orchestrator) = crossbeam_channel::unbounded();
    let (_, rx_explorer_to_planet) = crossbeam_channel::unbounded();
    let mut carbonium = create_carbonium(
        0,
        rx_orchestrator_to_planet,
        tx_planet_to_orchestrator,
        rx_explorer_to_planet,
    );

    let thread = thread::spawn(move || carbonium.run());

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::StartPlanetAI)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(matches!(
        res,
        Ok(PlanetToOrchestrator::StartPlanetAIResult { planet_id: 0 })
    ));

    for _ in 0..2 {
        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::Sunray(FORGE.generate_sunray()))
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::SunrayAck { planet_id: 0 })
        ));
    }

    // First you the already available Rocket, then build one on the fly.
    for _ in 0..2 {
        tx_orchestrator_to_planet
            .send(OrchestratorToPlanet::Asteroid(FORGE.generate_asteroid()))
            .unwrap();

        let res = rx_planet_to_orchestrator.recv();
        assert!(matches!(
            res,
            Ok(PlanetToOrchestrator::AsteroidAck {
                planet_id: 0,
                rocket: Some(_)
            })
        ));
    }

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::Asteroid(FORGE.generate_asteroid()))
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(matches!(
        res,
        Ok(PlanetToOrchestrator::AsteroidAck {
            planet_id: 0,
            rocket: None
        })
    ));

    tx_orchestrator_to_planet
        .send(OrchestratorToPlanet::KillPlanet)
        .unwrap();

    let res = rx_planet_to_orchestrator.recv();
    assert!(matches!(
        res,
        Ok(PlanetToOrchestrator::KillPlanetResult { planet_id: 0 })
    ));

    assert!(matches!(thread.join(), Ok(_)));
}
