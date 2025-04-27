use input_device::{Key, InputDeviceSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputDeviceSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    sim.key_down(Key::A)?;
    std::thread::sleep(std::time::Duration::from_secs(3));
    sim.key_up(Key::A)?;
    Ok(())
}
