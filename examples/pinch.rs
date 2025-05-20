use input_device::{InputSimulator, SimulationError};

pub fn main() -> Result<(), SimulationError> {
    let mut sim = InputSimulator::new()?;
    std::thread::sleep(std::time::Duration::from_secs(1));
    sim.touch_down(0, 300, 300)?;
    sim.touch_down(1, 600, 300)?;
    for i in 1..10 {
        sim.touch_move(0, 300 + i * 10, 300)?;
        sim.touch_move(1, 600 - i * 10, 300)?;
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    sim.touch_up(0)?;
    sim.touch_up(1)?;
    Ok(())
}
