use input_device::{InputSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    sim.touch_down(0, 200, 200)?;
    //std::thread::sleep(std::time::Duration::from_secs(3));
    sim.touch_move(0, 400, 400)?;
    //std::thread::sleep(std::time::Duration::from_secs(1));
    sim.touch_up(0)?;
    Ok(())
}
