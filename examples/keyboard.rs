use input_device::{InputSimulator, Key, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    sim.key_down(Key::LeftShift)?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    sim.key_down(Key::A)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_up(Key::A)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_down(Key::B)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_up(Key::B)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_down(Key::C)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_up(Key::C)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_down(Key::Dot)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    sim.key_up(Key::Dot)?;
    sim.key_up(Key::LeftShift)?;
    Ok(())
}
